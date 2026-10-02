use haven_domain::offer::{CurrencyCode, Offer, OfferId, Price, UserId};
use crate::DbPool;

pub async fn find_by_id(pool: &DbPool, offer_id: OfferId) -> sqlx::Result<Option<Offer>> {
    let o_id = offer_id.as_uuid();
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, title, description, price, currency, created_at, updated_at
        FROM offer
        WHERE id = $1
        "#,
        o_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Offer {
        id: OfferId(r.id),
        user_id: UserId(r.user_id),
        title: r.title,
        description: r.description,
        price: r.price.map(|p| Price::new(p).unwrap()),
        currency: r.currency.map(|c| CurrencyCode::parse(&c).unwrap()),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

/// Retrieves all offers for a given user. 
/// NOTE: Currently unbounded `fetch_all`. In v0.1.x, users have a small bounded number of offers.
/// As usage grows, an explicit limit/pagination strategy must be implemented per DB guardrails.
pub async fn find_by_user(pool: &DbPool, user_id: UserId) -> sqlx::Result<Vec<Offer>> {
    let u_id = user_id.as_uuid();
    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, title, description, price, currency, created_at, updated_at
        FROM offer
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        u_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Offer {
            id: OfferId(r.id),
            user_id: UserId(r.user_id),
            title: r.title,
            description: r.description,
            price: r.price.map(|p| Price::new(p).unwrap()),
            currency: r.currency.map(|c| CurrencyCode::parse(&c).unwrap()),
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}
