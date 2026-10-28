//! Persistent category, visibility and keyset guarantees.

use super::*;
use haven_domain::offer::{Category, OfferCursor, PublicCriteria, PublicFeedQuery};

#[sqlx::test(migrations = "../../migrations")]
async fn migration_backfills_test_rows_and_requires_known_categories(pool: PgPool) {
    let mut tx = pool.begin().await.unwrap();
    // Shadow the real table to exercise the migration against an existing row.
    sqlx::raw_sql("CREATE TEMP TABLE offer (id UUID PRIMARY KEY, image_count SMALLINT NOT NULL, created_at TIMESTAMPTZ NOT NULL);
        INSERT INTO offer VALUES (gen_random_uuid(), 1, NOW());").execute(&mut *tx).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../../../migrations/017_offer_category.sql"
    ))
    .execute(&mut *tx)
    .await
    .unwrap();
    let category: String = sqlx::query_scalar("SELECT category FROM offer")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(category, "other");
    let columns: (bool, bool) = sqlx::query_as("SELECT attnotnull, atthasdef FROM pg_attribute WHERE attrelid = 'pg_temp.offer'::regclass AND attname = 'category'")
        .fetch_one(&mut *tx).await.unwrap();
    assert_eq!(columns, (true, false));
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
#[allow(
    clippy::too_many_lines,
    reason = "Category intersection, paging and reclassification journey"
)]
async fn categories_persist_intersect_and_page_before_limit(pool: PgPool) {
    let account: Uuid = sqlx::query_scalar(
        "INSERT INTO account (email) VALUES ('category@example.com') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let person: Uuid = sqlx::query_scalar("INSERT INTO \"user\" (account_id, display_name, handle) VALUES ($1, 'Category Maker', 'category-maker') RETURNING id")
        .bind(account).fetch_one(&pool).await.unwrap();
    let person = UserId::from_uuid(person);
    let repo = PostgresOfferRepository::new(pool.clone());
    let images = images::PostgresOfferImageRepository::new(pool.clone());
    let mut food = Vec::new();
    for (title, category, visible) in [
        ("Market rice one", Category::FoodGroceries, true),
        ("Market rice two", Category::FoodGroceries, true),
        ("Market rice book", Category::BooksLearning, true),
        ("Market rice private", Category::FoodGroceries, false),
    ] {
        let offer = repo
            .create(
                person,
                IdempotencyKey::new(Uuid::new_v4()),
                &CreateOffer {
                    title: title.to_string(),
                    description: "Fresh stock from the local market.".to_string(),
                    price: Price::new(100).unwrap(),
                    currency: CurrencyCode::Ngn,
                    category,
                },
            )
            .await
            .unwrap()
            .into_inner();
        assert_eq!(offer.category, category);
        assert_eq!(
            repo.find_owned(offer.id, person)
                .await
                .unwrap()
                .unwrap()
                .category,
            category
        );
        assert_eq!(
            repo.find_by_slug(&offer.slug)
                .await
                .unwrap()
                .unwrap()
                .category,
            category
        );
        if visible {
            images
                .add_image(
                    offer.id,
                    person,
                    &haven_domain::image::ImagePublicId::parse(format!(
                        "offers/category/{}",
                        offer.id
                    ))
                    .unwrap(),
                    haven_domain::image::ImageDimensions::new(640, 480).unwrap(),
                )
                .await
                .unwrap();
            if category == Category::FoodGroceries {
                food.push(offer.id);
            }
        }
    }
    let criteria = PublicCriteria {
        category: Some(Category::FoodGroceries),
        title: Some(haven_domain::offer::SearchQuery::parse("rice").unwrap()),
        price: Some(
            haven_domain::offer::PriceFilter::new(
                CurrencyCode::Ngn,
                Some(Price::new(100).unwrap()),
                Some(Price::new(100).unwrap()),
            )
            .unwrap(),
        ),
    };
    let first = repo
        .list_public(&PublicFeedQuery::new(criteria.clone(), None, 1))
        .await
        .unwrap();
    assert_eq!(first.items.len(), 1);
    let cursor = OfferCursor::decode(first.next_cursor.as_deref().unwrap()).unwrap();
    let second = repo
        .list_public(&PublicFeedQuery::new(criteria.clone(), Some(cursor), 1))
        .await
        .unwrap();
    assert_eq!(second.items.len(), 1);
    assert!(second.next_cursor.is_none());
    let actual: std::collections::HashSet<_> = first
        .items
        .iter()
        .chain(second.items.iter())
        .map(|item| item.offer.id)
        .collect();
    assert_eq!(actual, food.iter().copied().collect());

    let id = food[0];
    assert!(
        sqlx::query("UPDATE offer SET category = 'unknown-topic' WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE offer SET category = NULL WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&pool)
            .await
            .is_err()
    );
    let updated = repo
        .update(
            id,
            person,
            &UpdateOffer {
                category: Some(Category::BooksLearning),
                title: None,
                description: None,
                price: None,
                currency: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.category, Category::BooksLearning);
    let remaining = repo
        .list_public(&PublicFeedQuery::new(criteria, None, 20))
        .await
        .unwrap();
    assert_eq!(remaining.items.len(), 1);
    assert!(remaining.items.iter().all(|item| item.offer.id != id));
    assert!(
        repo.find_by_user(person)
            .await
            .unwrap()
            .iter()
            .any(|offer| offer.id == id && offer.category == Category::BooksLearning)
    );
}
