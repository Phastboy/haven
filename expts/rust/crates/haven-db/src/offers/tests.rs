use super::*;
use haven_domain::offer::{CreateOffer, CurrencyCode, Price, UpdateOffer};
use haven_domain::ports::{IdempotencyKey, OfferRepository};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::test]
async fn test_update_patch_semantics() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@127.0.0.1:5433/haven_rust".to_string());

    let pool = match PgPool::connect(&db_url).await {
        Ok(p) => p,
        Err(_) => return, // Skip if DB is not available
    };

    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    let repo = PostgresOfferRepository::new(pool.clone());

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
    sqlx::query("INSERT INTO \"user\" (id, account_id) VALUES ($1, $2)")
        .bind(user_id.as_uuid())
        .bind(account_id)
        .execute(&pool)
        .await
        .unwrap();

    // 2. Create an offer
    let co = CreateOffer {
        title: "Original Title".to_string(),
        description: "Original Description".to_string(),
        price: Price::new(100).unwrap(),
        currency: CurrencyCode::parse("USD").unwrap(),
    };
    let offer = repo
        .create(user_id, IdempotencyKey::new(Uuid::new_v4()), &co)
        .await
        .unwrap();

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

    // 7. Seed multiple offers and test keyset feed pagination
    for i in 1..=5 {
        let req = CreateOffer {
            title: format!("Feed Offer {i}"),
            description: format!("Description for feed offer number {i}"),
            price: Price::new(10_i32.saturating_mul(i)).unwrap(),
            currency: CurrencyCode::parse("USD").unwrap(),
        };
        repo.create(user_id, IdempotencyKey::new(Uuid::new_v4()), &req)
            .await
            .unwrap();
    }

    // Fetch first page of 3 items
    let page1 = repo.list_public(None, 3).await.unwrap();
    assert_eq!(page1.items.len(), 3);
    assert!(page1.next_cursor.is_some());

    // Fetch second page using cursor
    let cursor_str = page1.next_cursor.unwrap();
    let cursor = haven_domain::offer::OfferCursor::decode(&cursor_str).unwrap();
    let page2 = repo.list_public(Some(&cursor), 3).await.unwrap();
    assert!(!page2.items.is_empty());

    // Ensure no overlap between page 1 and page 2
    let p1_ids: std::collections::HashSet<_> = page1.items.iter().map(|o| o.id).collect();
    for item in &page2.items {
        assert!(
            !p1_ids.contains(&item.id),
            "Keyset pagination must not duplicate items"
        );
    }
}
