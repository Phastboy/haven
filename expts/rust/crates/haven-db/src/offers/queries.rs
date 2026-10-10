//! Individual SQL query implementations for `OfferRepository`.

use crate::DbPool;
use haven_domain::offer::{
    CreateOffer, CurrencyCode, Offer, OfferId, OfferSlug, UpdateOffer, UserId,
};
use haven_domain::ports::{IdempotencyKey, RepoError};
use uuid::Uuid;

use super::rows::map_row;

pub(super) async fn find_owned(
    pool: &DbPool,
    offer_id: OfferId,
    user_id: UserId,
) -> Result<Option<Offer>, RepoError> {
    let o_id = offer_id.as_uuid();
    let u_id = user_id.as_uuid();
    let row = sqlx::query!(
        r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE id = $1 AND user_id = $2
            "#,
        o_id,
        u_id
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    row.map(|r| {
        map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        )
    })
    .transpose()
}

pub(super) async fn find_by_user(pool: &DbPool, user_id: UserId) -> Result<Vec<Offer>, RepoError> {
    let u_id = user_id.as_uuid();
    let rows = sqlx::query!(
        r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE user_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        u_id
    )
    .fetch_all(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let mut offers = Vec::with_capacity(rows.len());
    for r in rows {
        offers.push(map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        )?);
    }
    Ok(offers)
}

pub(super) async fn find_by_slug(
    pool: &DbPool,
    slug: &OfferSlug,
) -> Result<Option<Offer>, RepoError> {
    let slug_str = slug.as_str();
    let row = sqlx::query!(
        r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE slug = $1
            "#,
        slug_str
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    row.map(|r| {
        map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        )
    })
    .transpose()
}

pub(super) async fn create(
    pool: &DbPool,
    user_id: UserId,
    key: IdempotencyKey,
    create_offer: &CreateOffer,
) -> Result<Offer, RepoError> {
    let u_id = user_id.as_uuid();
    let price_val = create_offer.price.as_i32();
    let currency_val = create_offer.currency.as_str();
    let k = key.0;

    let offer_id = Uuid::new_v4();
    let mut suffix = String::new();
    for ch in offer_id.to_string().chars().take(8) {
        suffix.push(ch);
    }
    let slug = OfferSlug::from_title_and_suffix(&create_offer.title, &suffix)
        .map_err(|e| RepoError::Corrupt(e.to_string()))?;
    let slug_str = slug.as_str();

    let row = sqlx::query!(
        r#"
            INSERT INTO offer (id, user_id, slug, title, description, price, currency, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, idempotency_key) DO NOTHING
            RETURNING id, user_id, slug, title, description, price, currency, created_at, updated_at
            "#,
        offer_id,
        u_id,
        slug_str,
        create_offer.title,
        create_offer.description,
        price_val,
        currency_val,
        k
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    if let Some(r) = row {
        map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        )
    } else {
        let r = sqlx::query!(
            r#"
                SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
                FROM offer
                WHERE user_id = $1 AND idempotency_key = $2
                "#,
            u_id,
            k
        )
        .fetch_one(pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        )
    }
}

pub(super) async fn update(
    pool: &DbPool,
    offer_id: OfferId,
    user_id: UserId,
    update_offer: &UpdateOffer,
) -> Result<Offer, RepoError> {
    let o_id = offer_id.as_uuid();
    let u_id = user_id.as_uuid();
    let title_val = update_offer.title.as_deref();
    let desc_val = update_offer.description.as_deref();
    let price_val = update_offer.price.map(|p| p.as_i32());
    let currency_val = update_offer.currency.as_ref().map(CurrencyCode::as_str);

    let row = sqlx::query!(
        r#"
            UPDATE offer
            SET title = COALESCE($1, title),
                description = COALESCE($2, description),
                price = COALESCE($3, price),
                currency = COALESCE($4, currency),
                updated_at = NOW()
            WHERE id = $5 AND user_id = $6
            RETURNING id, user_id, slug, title, description, price, currency, created_at, updated_at
            "#,
        title_val,
        desc_val,
        price_val,
        currency_val,
        o_id,
        u_id
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    match row {
        Some(r) => map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.created_at,
            r.updated_at,
        ),
        None => Err(RepoError::NotFound),
    }
}

pub(super) async fn delete(
    pool: &DbPool,
    offer_id: OfferId,
    user_id: UserId,
) -> Result<(), RepoError> {
    let o_id = offer_id.as_uuid();
    let u_id = user_id.as_uuid();
    let rows_affected = sqlx::query!(
        r#"
            DELETE FROM offer
            WHERE id = $1 AND user_id = $2
            "#,
        o_id,
        u_id
    )
    .execute(pool)
    .await
    .map_err(crate::map_sqlx_err)?
    .rows_affected();

    if rows_affected == 0 {
        Err(RepoError::NotFound)
    } else {
        Ok(())
    }
}
