//! Individual SQL query implementations for `OfferRepository`.

use crate::DbPool;
use haven_domain::image::{ImageDimensions, ImageId, ImagePosition, ImagePublicId, OfferImage};
use haven_domain::offer::{
    CreateOffer, CurrencyCode, Offer, OfferId, OfferSlug, UpdateOffer, UserId,
};
use haven_domain::ports::{CreationStatus, IdempotencyKey, RepoError};
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
            SELECT id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at
            FROM offer
            WHERE id = $1 AND user_id = $2
            "#,
        o_id,
        u_id
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    if let Some(r) = row {
        let images = super::images::fetch_images_for_offer(pool, offer_id).await?;
        let offer = map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.image_count,
            images,
            r.created_at,
            r.updated_at,
        )?;
        Ok(Some(offer))
    } else {
        Ok(None)
    }
}

/// Fetches all offers owned by a user, hydrated with their position 1 cover images via LEFT JOIN.
pub(super) async fn find_by_user(pool: &DbPool, user_id: UserId) -> Result<Vec<Offer>, RepoError> {
    let u_id = user_id.as_uuid();
    let rows = sqlx::query!(
        r#"
            SELECT 
                o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.image_count, o.created_at, o.updated_at,
                img.id AS "img_id?", img.public_id AS "img_public_id?", img.position AS "img_position?", 
                img.width AS "img_width?", img.height AS "img_height?", img.created_at AS "img_created_at?"
            FROM offer o
            LEFT JOIN offer_image img ON img.offer_id = o.id AND img.position = 1
            WHERE o.user_id = $1
            ORDER BY o.created_at DESC, o.id DESC
            "#,
        u_id
    )
    .fetch_all(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let mut offers = Vec::with_capacity(rows.len());
    for r in rows {
        let images =
            if let (Some(img_id), Some(public_id), Some(pos), Some(w), Some(h), Some(c_at)) = (
                r.img_id,
                r.img_public_id,
                r.img_position,
                r.img_width,
                r.img_height,
                r.img_created_at,
            ) {
                let pos_u8 = u8::try_from(pos)
                    .map_err(|_| RepoError::Corrupt("invalid image position in database".into()))?;
                let position =
                    ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;
                let public_id = ImagePublicId::parse(&public_id)
                    .map_err(|e| RepoError::Corrupt(e.to_string()))?;
                let width = u32::try_from(w)
                    .map_err(|_| RepoError::Corrupt("invalid image width in database".into()))?;
                let height = u32::try_from(h)
                    .map_err(|_| RepoError::Corrupt("invalid image height in database".into()))?;
                let dimensions = ImageDimensions::new(width, height)
                    .map_err(|e| RepoError::Corrupt(e.to_string()))?;
                vec![OfferImage {
                    id: ImageId::from_uuid(img_id),
                    offer_id: OfferId::from_uuid(r.id),
                    public_id,
                    position,
                    dimensions,
                    created_at: c_at,
                }]
            } else {
                Vec::new()
            };

        offers.push(map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.image_count,
            images,
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
            SELECT id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at
            FROM offer
            WHERE slug = $1
            "#,
        slug_str
    )
    .fetch_optional(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    if let Some(r) = row {
        let images = super::images::fetch_images_for_offer(pool, OfferId::from_uuid(r.id)).await?;
        let offer = map_row(
            r.id,
            r.user_id,
            r.slug,
            r.title,
            r.description,
            r.price,
            &r.currency,
            r.image_count,
            images,
            r.created_at,
            r.updated_at,
        )?;
        Ok(Some(offer))
    } else {
        Ok(None)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Internal database query execution helper"
)]
/// Inserts an offer idempotently using a single atomic CTE, returning whether it was newly created or replayed.
pub(super) async fn create(
    pool: &DbPool,
    user_id: UserId,
    key: IdempotencyKey,
    create_offer: &CreateOffer,
) -> Result<CreationStatus<Offer>, RepoError> {
    let u_id = user_id.as_uuid();
    let price_val = create_offer.price.as_i32();
    let currency_val = create_offer.currency.as_str();
    let k = key.as_uuid();

    let offer_id = Uuid::new_v4();
    let mut suffix = String::new();
    for ch in offer_id.to_string().chars().take(8) {
        suffix.push(ch);
    }
    let slug = OfferSlug::from_title_and_suffix(&create_offer.title, &suffix)
        .map_err(|e| RepoError::Corrupt(e.to_string()))?;
    let slug_str = slug.as_str();

    let r = sqlx::query!(
        r#"
        WITH inserted AS (
            INSERT INTO offer (id, user_id, slug, title, description, price, currency, idempotency_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_id, idempotency_key) DO NOTHING
            RETURNING id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at, true AS is_new
        )
        SELECT 
            id AS "id!", 
            user_id AS "user_id!", 
            slug AS "slug!", 
            title AS "title!", 
            description AS "description!", 
            price AS "price!", 
            currency AS "currency!", 
            image_count AS "image_count!", 
            created_at AS "created_at!", 
            updated_at AS "updated_at!", 
            is_new AS "is_new!"
        FROM inserted
        UNION ALL
        SELECT 
            id, 
            user_id, 
            slug, 
            title, 
            description, 
            price, 
            currency, 
            image_count, 
            created_at, 
            updated_at, 
            false AS is_new
        FROM offer
        WHERE user_id = $2 AND idempotency_key = $8
          AND NOT EXISTS (SELECT 1 FROM inserted)
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
    .fetch_one(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let offer = map_row(
        r.id,
        r.user_id,
        r.slug,
        r.title,
        r.description,
        r.price,
        &r.currency,
        r.image_count,
        Vec::new(),
        r.created_at,
        r.updated_at,
    )?;

    if r.is_new {
        Ok(CreationStatus::Created(offer))
    } else {
        Ok(CreationStatus::Replayed(offer))
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Internal database query execution helper"
)]
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
            RETURNING id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at
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
        Some(r) => {
            let images = super::images::fetch_images_for_offer(pool, offer_id).await?;
            map_row(
                r.id,
                r.user_id,
                r.slug,
                r.title,
                r.description,
                r.price,
                &r.currency,
                r.image_count,
                images,
                r.created_at,
                r.updated_at,
            )
        }
        None => Err(RepoError::NotFound {
            entity: "offer",
            id: offer_id.to_string(),
        }),
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
        Err(RepoError::NotFound {
            entity: "offer",
            id: offer_id.to_string(),
        })
    } else {
        Ok(())
    }
}
