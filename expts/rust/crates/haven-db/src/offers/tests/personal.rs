//! Personal membership, activity ordering, facets and participant isolation.

use super::*;
use haven_domain::offer::{
    Category, PersonalCursor, PersonalFeedQuery, PersonalScope, PriceFilter, PublicCriteria,
};

async fn person(pool: &PgPool, number: i32) -> UserId {
    let account: Uuid = sqlx::query_scalar("INSERT INTO account (email) VALUES ($1) RETURNING id")
        .bind(format!("personal-{number}@example.com"))
        .fetch_one(pool)
        .await
        .unwrap();
    let person: Uuid = sqlx::query_scalar(
        "INSERT INTO \"user\" (account_id,display_name,handle) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(account)
    .bind(format!("Person {number}"))
    .bind(format!("personal-{number}"))
    .fetch_one(pool)
    .await
    .unwrap();
    UserId::from_uuid(person)
}

async fn offer(
    repo: &PostgresOfferRepository,
    owner: UserId,
    number: i32,
) -> haven_domain::offer::Offer {
    repo.create(
        owner,
        IdempotencyKey::new(Uuid::new_v4()),
        &CreateOffer {
            category: Category::FoodGroceries,
            title: format!("Market rice {number}"),
            description: "Fresh rice offered by the bag.".to_string(),
            price: Price::new(if number % 2 == 0 { 100 } else { 300 }).unwrap(),
            currency: CurrencyCode::Ngn,
        },
    )
    .await
    .unwrap()
    .into_inner()
}

async fn visible(pool: &PgPool, id: haven_domain::offer::OfferId) {
    sqlx::query("INSERT INTO offer_image (offer_id,public_id,position,width,height) VALUES ($1,$2,1,640,480)")
        .bind(id.as_uuid()).bind(format!("offers/personal/{id}")).execute(pool).await.unwrap();
    sqlx::query("UPDATE offer SET image_count=1 WHERE id=$1")
        .bind(id.as_uuid())
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
#[allow(
    clippy::too_many_lines,
    reason = "Saved activity, facets and two-page isolation journey"
)]
async fn saved_filters_before_limit_and_orders_by_save_activity(pool: PgPool) {
    let owner = person(&pool, 0).await;
    let actor = person(&pool, 1).await;
    let other = person(&pool, 2).await;
    let repo = PostgresOfferRepository::new(pool.clone());
    let mut expected = Vec::new();
    for number in 0..50 {
        let offer = offer(&repo, owner, number).await;
        visible(&pool, offer.id).await;
        sqlx::query("INSERT INTO saved_offer (user_id,offer_id,created_at) VALUES ($1,$2,'2020-01-01'::timestamptz + $3::int * interval '1 second')")
            .bind(actor.as_uuid()).bind(offer.id.as_uuid()).bind(number).execute(&pool).await.unwrap();
        if number % 2 == 0 {
            expected.push(offer.id);
        }
    }
    let private = offer(&repo, owner, 100).await;
    sqlx::query("INSERT INTO saved_offer (user_id,offer_id) VALUES ($1,$2)")
        .bind(actor.as_uuid())
        .bind(private.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let unrelated = offer(&repo, owner, 102).await;
    visible(&pool, unrelated.id).await;
    sqlx::query("INSERT INTO saved_offer (user_id,offer_id) VALUES ($1,$2)")
        .bind(other.as_uuid())
        .bind(unrelated.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let criteria = PublicCriteria {
        category: Some(Category::FoodGroceries),
        title: Some(haven_domain::offer::SearchQuery::parse("rice").unwrap()),
        price: Some(
            PriceFilter::new(CurrencyCode::Ngn, None, Some(Price::new(100).unwrap())).unwrap(),
        ),
    };
    let scope = PersonalScope::Saved(actor);
    let first = repo
        .list_personal(&PersonalFeedQuery::new(scope, criteria.clone()))
        .await
        .unwrap();
    assert_eq!(first.items.len(), 20);
    let boundary =
        PersonalCursor::decode(first.next_cursor.as_deref().unwrap(), scope, &criteria).unwrap();
    let second = repo
        .list_personal(
            &PersonalFeedQuery::new(scope, criteria)
                .after(boundary)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.items.len(), 5);
    assert!(second.next_cursor.is_none());
    let ids: Vec<_> = first
        .items
        .iter()
        .chain(second.items.iter())
        .map(|item| item.offer.id)
        .collect();
    expected.reverse();
    assert_eq!(ids, expected);
    assert!(!ids.contains(&private.id));
    assert!(!ids.contains(&unrelated.id));
    sqlx::query("DELETE FROM saved_offer WHERE user_id=$1 AND offer_id=$2")
        .bind(actor.as_uuid())
        .bind(first.items[0].offer.id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let refreshed = repo
        .list_personal(&PersonalFeedQuery::new(scope, PublicCriteria::default()))
        .await
        .unwrap();
    assert!(
        refreshed
            .items
            .iter()
            .all(|item| item.offer.id != first.items[0].offer.id)
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[allow(
    clippy::too_many_lines,
    reason = "Qualifying inquiries, replies, repeated tags and invalid conversations"
)]
async fn inquired_uses_latest_qualifying_sent_message_and_deduplicates(pool: PgPool) {
    let owner = person(&pool, 0).await;
    let actor = person(&pool, 1).await;
    let other = person(&pool, 2).await;
    let repo = PostgresOfferRepository::new(pool.clone());
    let old = offer(&repo, owner, 0).await;
    let newer = offer(&repo, owner, 2).await;
    let received = offer(&repo, owner, 4).await;
    let private = offer(&repo, owner, 6).await;
    let own = offer(&repo, actor, 8).await;
    let unqualified = offer(&repo, owner, 10).await;
    for id in [old.id, newer.id, received.id, own.id, unqualified.id] {
        visible(&pool, id).await;
    }
    let conversation: Uuid = sqlx::query_scalar("INSERT INTO conversation (user_low,user_high) VALUES (LEAST($1::uuid,$2::uuid),GREATEST($1::uuid,$2::uuid)) RETURNING id")
        .bind(owner.as_uuid()).bind(actor.as_uuid()).fetch_one(&pool).await.unwrap();
    let unrelated: Uuid = sqlx::query_scalar("INSERT INTO conversation (user_low,user_high) VALUES (LEAST($1::uuid,$2::uuid),GREATEST($1::uuid,$2::uuid)) RETURNING id")
        .bind(owner.as_uuid()).bind(other.as_uuid()).fetch_one(&pool).await.unwrap();
    for (conv, sender, id, seconds) in [
        (conversation, actor, old.id, 1),
        (conversation, actor, old.id, 10),
        (conversation, actor, newer.id, 5),
        (conversation, owner, received.id, 99),
        (conversation, actor, own.id, 99),
        (conversation, actor, private.id, 99),
        (unrelated, actor, unqualified.id, 100),
        (unrelated, actor, old.id, 200),
        (unrelated, other, old.id, 300),
    ] {
        sqlx::query("INSERT INTO message (conversation_id,sender_id,offer_id,body,created_at) VALUES ($1,$2,$3,'Interested in this offer','2020-01-01'::timestamptz + $4::int * interval '1 second')")
            .bind(conv).bind(sender.as_uuid()).bind(id.as_uuid()).bind(seconds).execute(&pool).await.unwrap();
    }
    let scope = PersonalScope::Inquired(actor);
    let page = repo
        .list_personal(&PersonalFeedQuery::new(scope, PublicCriteria::default()))
        .await
        .unwrap();
    assert_eq!(
        page.items
            .iter()
            .map(|item| item.offer.id)
            .collect::<Vec<_>>(),
        [old.id, newer.id]
    );
    assert!(page.next_cursor.is_none());
    let other_page = repo
        .list_personal(&PersonalFeedQuery::new(
            PersonalScope::Inquired(owner),
            PublicCriteria::default(),
        ))
        .await
        .unwrap();
    assert!(
        other_page
            .items
            .iter()
            .all(|item| item.offer.user_id != owner)
    );
}
