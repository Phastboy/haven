use haven_domain::offer::{CurrencyCode, Offer, OfferId, Price, UpdateOffer, UserId};
use crate::DbPool;

pub async fn update(
    pool: &DbPool,
    offer_id: OfferId,
    update_offer: &UpdateOffer,
) -> sqlx::Result<Option<Offer>> {
    let o_id = offer_id.as_uuid();
    let price_val = update_offer.price.map(|p| p.as_i32());
    let currency_val = update_offer.currency.as_ref().map(|c| c.as_str());

    let row = sqlx::query!(
        r#"
        UPDATE offer
        SET title = $2, description = $3, price = $4, currency = $5, updated_at = now()
        WHERE id = $1
        RETURNING id, user_id, title, description, price, currency, created_at, updated_at
        "#,
        o_id,
        update_offer.title,
        update_offer.description,
        price_val,
        currency_val as Option<&str>
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
