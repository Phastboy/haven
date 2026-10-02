use haven_domain::offer::{CreateOffer, CurrencyCode, Offer, OfferId, Price, UpdateOffer, UserId};

use crate::DbPool;

pub async fn create(
    pool: &DbPool,
    user_id: UserId,
    create_offer: &CreateOffer,
) -> sqlx::Result<Offer> {
    let u_id = user_id.as_uuid();
    let price_val = create_offer.price.map(|p| p.as_i32());
    let currency_val = create_offer.currency.as_ref().map(|c| c.as_str());

    let row = sqlx::query!(
        r#"
        INSERT INTO offer (user_id, title, description, price, currency)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, title, description, price, currency, created_at, updated_at
        "#,
        u_id,
        create_offer.title,
        create_offer.description,
        price_val,
        currency_val as Option<&str>
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

pub async fn delete(pool: &DbPool, offer_id: OfferId) -> sqlx::Result<()> {
    let o_id = offer_id.as_uuid();
    sqlx::query!(
        r#"
        DELETE FROM offer
        WHERE id = $1
        "#,
        o_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
