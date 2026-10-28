use super::*;
use haven_domain::offer::{CreateOffer, CurrencyCode, Price, UpdateOffer};
use haven_domain::ports::{IdempotencyKey, OfferImageRepository, OfferRepository};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
#[allow(clippy::too_many_lines, reason = "Comprehensive patch semantics test")]
async fn test_update_patch_semantics(pool: PgPool) {
    let repo = PostgresOfferRepository::new(pool.clone());
    let img_repo = images::PostgresOfferImageRepository::new(pool.clone());
    let dims = haven_domain::image::ImageDimensions::new(800, 600).unwrap();

    // 1. Create a dummy account and user
    let account_id = Uuid::new_v4();
    let email = format!("{account_id}@example.com");
    sqlx::query("INSERT INTO account (id, email) VALUES ($1, $2)")
        .bind(account_id)
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();

    let user_id = UserId::from_uuid(Uuid::new_v4());
    let handle = format!("user-{}", &user_id.as_uuid().to_string()[..6]);
    sqlx::query(
        "INSERT INTO \"user\" (id, account_id, display_name, handle) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id.as_uuid())
    .bind(account_id)
    .bind("Test User")
    .bind(&handle)
    .execute(&pool)
    .await
    .unwrap();

    // 2. Create an offer
    let key = IdempotencyKey::new(Uuid::new_v4());
    let co = CreateOffer {
        title: "Original Title".to_string(),
        description: "Original Description".to_string(),
        price: Price::new(100).unwrap(),
        currency: CurrencyCode::parse("USD").unwrap(),
    };
    let status = repo.create(user_id, key, &co).await.unwrap();
    assert!(status.is_new());
    let offer = status.into_inner();

    // Verify idempotent replay
    let replay_status = repo.create(user_id, key, &co).await.unwrap();
    assert!(!replay_status.is_new());
    assert_eq!(replay_status.into_inner().id, offer.id);

    assert_eq!(offer.price.as_i32(), 100);
    assert_eq!(offer.currency.as_str(), "USD");
    assert_eq!(offer.description, "Original Description");

    // 3. Update title, leave description, price and currency unchanged (None)
    let uo1 = UpdateOffer {
        title: Some("New Title".to_string()),
        description: None,
        price: None,
        currency: None,
    };
    let offer1 = repo.update(offer.id, user_id, &uo1).await.unwrap();
    assert_eq!(offer1.title, "New Title");
    assert_eq!(offer1.description, "Original Description"); // Unchanged
    assert_eq!(offer1.price.as_i32(), 100); // Unchanged
    assert_eq!(offer1.currency.as_str(), "USD"); // Unchanged

    // 4. Update price only
    let uo2 = UpdateOffer {
        title: None,
        description: None,
        price: Some(Price::new(200).unwrap()),
        currency: None,
    };
    let offer2 = repo.update(offer.id, user_id, &uo2).await.unwrap();
    assert_eq!(offer2.title, "New Title"); // Unchanged
    assert_eq!(offer2.description, "Original Description"); // Unchanged
    assert_eq!(offer2.price.as_i32(), 200);
    assert_eq!(offer2.currency.as_str(), "USD"); // Unchanged

    // 5. Update currency and description
    let uo3 = UpdateOffer {
        title: None,
        description: Some("Updated Description".to_string()),
        price: None,
        currency: Some(CurrencyCode::parse("NGN").unwrap()),
    };
    let offer3 = repo.update(offer.id, user_id, &uo3).await.unwrap();
    assert_eq!(offer3.title, "New Title"); // Unchanged
    assert_eq!(offer3.description, "Updated Description");
    assert_eq!(offer3.price.as_i32(), 200); // Unchanged
    assert_eq!(offer3.currency.as_str(), "NGN");

    // 6. Test find_by_slug
    let found_by_slug = repo.find_by_slug(&offer.slug).await.unwrap();
    assert!(found_by_slug.is_some());
    let found = found_by_slug.unwrap();
    assert_eq!(found.id, offer.id);
    assert_eq!(found.slug, offer.slug);

    // 7. Seed multiple offers with images and test keyset feed pagination
    for i in 1..=5 {
        let req = CreateOffer {
            title: format!("Feed Offer {i}"),
            description: format!("Description for feed offer number {i}"),
            price: Price::new(10_i32.saturating_mul(i)).unwrap(),
            currency: CurrencyCode::parse("USD").unwrap(),
        };
        let created = repo
            .create(user_id, IdempotencyKey::new(Uuid::new_v4()), &req)
            .await
            .unwrap()
            .into_inner();
        let item_pub_id = haven_domain::image::ImagePublicId::parse(format!(
            "v12345/haven/offers/test_img_{i}_{}",
            Uuid::new_v4().simple()
        ))
        .unwrap();
        img_repo
            .add_image(created.id, user_id, &item_pub_id, dims)
            .await
            .unwrap();
    }

    // Fetch first page of 3 items
    let page1 = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria::default(),
            None,
            3,
        ))
        .await
        .unwrap();
    assert_eq!(page1.items.len(), 3);
    assert!(page1.next_cursor.is_some());

    // Fetch second page using cursor
    let cursor_str = page1.next_cursor.unwrap();
    let cursor = haven_domain::offer::OfferCursor::decode(&cursor_str).unwrap();
    let page2 = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria::default(),
            Some(cursor),
            3,
        ))
        .await
        .unwrap();
    assert!(!page2.items.is_empty());

    // Ensure no overlap between page 1 and page 2
    let p1_ids: std::collections::HashSet<_> = page1.items.iter().map(|o| o.offer.id).collect();
    for item in &page2.items {
        assert!(
            !p1_ids.contains(&item.offer.id),
            "Keyset pagination must not duplicate items"
        );
        assert_eq!(item.maker_display_name.as_str(), "Test User");
        assert_eq!(item.maker_handle.as_str(), &handle);
    }

    // 8. Test list_by_user pagination
    let user_page = repo.list_by_user(user_id, None, 3).await.unwrap();
    assert_eq!(user_page.items.len(), 3);
    assert!(user_page.next_cursor.is_some());

    // 9. Test OfferImageRepository operations
    let pub_id1 = haven_domain::image::ImagePublicId::parse(format!(
        "v12345/haven/offers/test_img_sec9_1_{}",
        Uuid::new_v4().simple()
    ))
    .unwrap();
    let img1 = img_repo
        .add_image(offer.id, user_id, &pub_id1, dims)
        .await
        .unwrap();
    assert_eq!(img1.position.as_u8(), 1);

    let pub_id2 = haven_domain::image::ImagePublicId::parse(format!(
        "v12345/haven/offers/test_img_sec9_2_{}",
        Uuid::new_v4().simple()
    ))
    .unwrap();
    let img2 = img_repo
        .add_image(offer.id, user_id, &pub_id2, dims)
        .await
        .unwrap();
    assert_eq!(img2.position.as_u8(), 2);

    let images_list = img_repo.find_by_offer(offer.id).await.unwrap();
    assert_eq!(images_list.len(), 2);
    assert_eq!(images_list[0].id, img1.id);
    assert_eq!(images_list[1].id, img2.id);

    // Reorder images
    let reordered = img_repo
        .reorder_images(offer.id, user_id, &[img2.id, img1.id])
        .await
        .unwrap();
    assert_eq!(reordered[0].id, img2.id);
    assert_eq!(reordered[0].position.as_u8(), 1);
    assert_eq!(reordered[1].id, img1.id);
    assert_eq!(reordered[1].position.as_u8(), 2);

    // Remove image
    let removed_pub_id = img_repo
        .remove_image(offer.id, user_id, img2.id)
        .await
        .unwrap();
    assert_eq!(removed_pub_id.as_str(), pub_id2.as_str());

    let remaining = img_repo.find_by_offer(offer.id).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, img1.id);
    assert_eq!(remaining[0].position.as_u8(), 1);

    // Verify outbox trigger captured the deleted image public ID
    let outbox_rows = sqlx::query("SELECT public_id FROM image_purge_outbox WHERE public_id = $1")
        .bind(removed_pub_id.as_str())
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(outbox_rows.len(), 1);

    // Verify minimum image invariant: cannot remove the last image
    let conflict_err = img_repo.remove_image(offer.id, user_id, img1.id).await;
    assert!(
        matches!(
            conflict_err,
            Err(haven_domain::ports::RepoError::Conflict { .. })
        ),
        "Deleting the last image must return Conflict"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_offers_search_and_price_filtering(pool: PgPool) {
    let repo = PostgresOfferRepository::new(pool.clone());
    let img_repo = images::PostgresOfferImageRepository::new(pool.clone());

    let account_id = Uuid::new_v4();
    let email = format!("{account_id}@example.com");
    sqlx::query("INSERT INTO account (id, email) VALUES ($1, $2)")
        .bind(account_id)
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();

    let user_id = UserId::from_uuid(Uuid::new_v4());
    let handle = format!("user-{}", &user_id.as_uuid().to_string()[..6]);
    sqlx::query(
        "INSERT INTO \"user\" (id, account_id, display_name, handle) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id.as_uuid())
    .bind(account_id)
    .bind("Test User")
    .bind(&handle)
    .execute(&pool)
    .await
    .unwrap();

    // Create 3 offers with distinct titles, prices, and currencies
    let o1 = repo
        .create(
            user_id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Vintage Wooden Lamp".to_string(),
                description: "Handcrafted wooden desk lamp in great condition.".to_string(),
                price: Price::new(5000).unwrap(),
                currency: CurrencyCode::Ngn,
            },
        )
        .await
        .unwrap()
        .into_inner();

    let o2 = repo
        .create(
            user_id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Free Wooden Chair".to_string(),
                description: "Sturdy wooden dining chair, free for pickup.".to_string(),
                price: Price::ZERO,
                currency: CurrencyCode::Ngn,
            },
        )
        .await
        .unwrap()
        .into_inner();

    let o3 = repo
        .create(
            user_id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Modern Glass Desk".to_string(),
                description: "Sleek glass computer desk in pristine shape.".to_string(),
                price: Price::new(15000).unwrap(),
                currency: CurrencyCode::Usd,
            },
        )
        .await
        .unwrap()
        .into_inner();

    // Attach cover images so all 3 offers are public (image_count >= 1)
    let dims = haven_domain::image::ImageDimensions::new(800, 600).unwrap();
    for off in [&o1, &o2, &o3] {
        let pub_id = haven_domain::image::ImagePublicId::parse(format!(
            "v12345/haven/offers/test_search_{}",
            Uuid::new_v4().simple()
        ))
        .unwrap();
        img_repo
            .add_image(off.id, user_id, &pub_id, dims)
            .await
            .unwrap();
    }

    // 1. Title search matching "Wooden" -> should return o1 and o2, but not o3
    let search_title = haven_domain::offer::SearchQuery::parse("Wooden").unwrap();
    let res_title = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria {
                title: Some(search_title),
                price: None,
            },
            None,
            10,
        ))
        .await
        .unwrap();
    let title_ids: Vec<_> = res_title.items.iter().map(|i| i.offer.id).collect();
    assert!(title_ids.contains(&o1.id));
    assert!(title_ids.contains(&o2.id));
    assert!(!title_ids.contains(&o3.id));

    // 2. Free price filter (price = 0, currency = NGN) -> should return o2 only
    let free_filter = haven_domain::offer::PriceFilter::free(CurrencyCode::Ngn);
    let res_free = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria {
                title: None,
                price: Some(free_filter),
            },
            None,
            10,
        ))
        .await
        .unwrap();
    let free_ids: Vec<_> = res_free.items.iter().map(|i| i.offer.id).collect();
    assert!(free_ids.contains(&o2.id));
    assert!(!free_ids.contains(&o1.id));
    assert!(!free_ids.contains(&o3.id));

    // 3. Price range filter (NGN 1000 to 10000) -> should return o1 only
    let range_filter = haven_domain::offer::PriceFilter::new(
        CurrencyCode::Ngn,
        Some(Price::new(1000).unwrap()),
        Some(Price::new(10000).unwrap()),
    )
    .unwrap();
    let res_range = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria {
                title: None,
                price: Some(range_filter),
            },
            None,
            10,
        ))
        .await
        .unwrap();
    let range_ids: Vec<_> = res_range.items.iter().map(|i| i.offer.id).collect();
    assert!(range_ids.contains(&o1.id));
    assert!(!range_ids.contains(&o2.id));
    assert!(!range_ids.contains(&o3.id));

    // 4. Combined title search ("Wooden") + Free filter -> should return o2 only
    let res_combined = repo
        .list_public(&haven_domain::offer::PublicFeedQuery::new(
            haven_domain::offer::PublicCriteria {
                title: Some(haven_domain::offer::SearchQuery::parse("Wooden").unwrap()),
                price: Some(free_filter),
            },
            None,
            10,
        ))
        .await
        .unwrap();
    let combined_ids: Vec<_> = res_combined.items.iter().map(|i| i.offer.id).collect();
    assert_eq!(combined_ids, vec![o2.id]);
}
