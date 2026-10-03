use haven_domain::offer::{CreateOffer, CurrencyCode, Offer, OfferId, Price, UserId};
use crate::DbPool;

pub async fn create(
    pool: &DbPool,
    user_id: UserId,
    create_offer: &CreateOffer,
    idempotency_key: uuid::Uuid,
) -> sqlx::Result<Offer> {
    let u_id = user_id.as_uuid();
    let price_val = create_offer.price.map(|p| p.as_i32());
    let currency_val = create_offer.currency.as_ref().map(|c| c.as_str());

    let row = sqlx::query!(
        r#"
        INSERT INTO offer (user_id, title, description, price, currency, idempotency_key)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, idempotency_key)
        DO UPDATE SET title = EXCLUDED.title 
        RETURNING id, user_id, title, description, price, currency, created_at, updated_at
        "#,
        u_id,
        create_offer.title,
        create_offer.description,
        price_val,
        currency_val as Option<&str>,
        idempotency_key
    )
    .fetch_one(pool)
    .await?;

    Ok(Offer {
        id: OfferId(row.id),
        user_id: UserId(row.user_id),
        title: row.title,
        description: row.description,
        price: row.price.map(|p| Price::new(p).unwrap()),
        currency: row.currency.map(|c| CurrencyCode::parse(&c).unwrap()),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}
