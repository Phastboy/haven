#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "database test setup and assertions"
)]

use super::*;
use haven_domain::{
    offer::{CreateOffer, CurrencyCode, Price},
    ports::{IdempotencyKey, OfferRepository},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn create_person(pool: &PgPool, label: &str) -> UserId {
    let account_id = Uuid::new_v4();
    let person_id = UserId::generate();
    let handle = format!("{label}-{}", &person_id.as_uuid().to_string()[..6]);
    sqlx::query("INSERT INTO account (id, email) VALUES ($1, $2)")
        .bind(account_id)
        .bind(format!("{account_id}@example.com"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO \"user\" (id, account_id, display_name, handle) VALUES ($1, $2, $3, $4)",
    )
    .bind(person_id.as_uuid())
    .bind(account_id)
    .bind(label)
    .bind(handle)
    .execute(pool)
    .await
    .unwrap();
    person_id
}

#[sqlx::test(migrations = "../../migrations")]
async fn saved_offers_and_follows_are_idempotent_and_keyset_paginated(pool: PgPool) {
    let owner_id = create_person(&pool, "owner").await;
    let followed_id = create_person(&pool, "followed").await;
    let offers = crate::offers::PostgresOfferRepository::new(pool.clone());
    let offer = offers
        .create(
            owner_id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                category: haven_domain::offer::Category::Other,
                title: "Saved offer".into(),
                description: "An offer to save".into(),
                price: Price::new(100).unwrap(),
                currency: CurrencyCode::parse("USD").unwrap(),
            },
        )
        .await
        .unwrap()
        .into_inner();
    sqlx::query("UPDATE offer SET image_count = 1 WHERE id = $1")
        .bind(offer.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let saved = PostgresSavedOfferRepository::new(pool.clone());
    saved.save(owner_id, offer.id).await.unwrap();
    let first_save = saved.list(owner_id, None, 1).await.unwrap();
    let saved_at = first_save.items[0].created_at;
    saved.save(owner_id, offer.id).await.unwrap();
    assert!(saved.is_saved(owner_id, offer.id).await.unwrap());
    assert_eq!(
        saved.saved_among(owner_id, &[offer.id]).await.unwrap(),
        [offer.id].into_iter().collect()
    );
    assert!(saved.saved_among(owner_id, &[]).await.unwrap().is_empty());
    let page = saved.list(owner_id, None, 1).await.unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].created_at, saved_at);
    assert_eq!(
        page.items[0].offer.as_ref().map(|item| item.id),
        Some(offer.id)
    );

    sqlx::query("UPDATE offer SET image_count = 0 WHERE id = $1")
        .bind(offer.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let unavailable = saved.list(owner_id, None, 1).await.unwrap();
    assert_eq!(unavailable.items.len(), 1);
    assert!(unavailable.items[0].offer.is_none());

    let follows = PostgresFollowRepository::new(pool.clone());
    assert!(follows.follow(owner_id, owner_id).await.is_err());
    follows.follow(owner_id, followed_id).await.unwrap();
    let first_follow = follows.list(owner_id, None, 1).await.unwrap();
    let followed_at = first_follow.items[0].created_at;
    follows.follow(owner_id, followed_id).await.unwrap();
    assert!(follows.is_following(owner_id, followed_id).await.unwrap());
    let page = follows.list(owner_id, None, 1).await.unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].created_at, followed_at);
    assert_eq!(page.items[0].person.id, followed_id);

    offers.delete(offer.id, owner_id).await.unwrap();
    assert!(
        saved
            .list(owner_id, None, 1)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    saved.unsave(owner_id, offer.id).await.unwrap();
    saved.unsave(owner_id, offer.id).await.unwrap();
    follows.unfollow(owner_id, followed_id).await.unwrap();
    follows.unfollow(owner_id, followed_id).await.unwrap();
    assert!(
        follows
            .list(owner_id, None, 1)
            .await
            .unwrap()
            .items
            .is_empty()
    );
}
