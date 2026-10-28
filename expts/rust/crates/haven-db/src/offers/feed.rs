//! Public offer feed keyset pagination query.

use crate::DbPool;
use haven_domain::offer::{OfferCursor, OfferPage, PublicOffer, PublicOfferPage, UserId};
use haven_domain::ports::RepoError;
use haven_domain::user::{DisplayName, ProfileHandle};

use super::rows::map_row;

/// Query parameters for fetching a user's offer feed.
pub(super) struct UserFeedQuery<'a> {
    pub user_id: UserId,
    pub cursor: Option<&'a OfferCursor>,
    pub limit: usize,
}

struct UserOfferDbRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub currency: String,
    pub image_count: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct PublicOfferDbRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub currency: String,
    pub image_count: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub display_name: String,
    pub handle: String,
}

fn map_public_row(
    r: PublicOfferDbRow,
    images: Vec<haven_domain::image::OfferImage>,
) -> Result<PublicOffer, RepoError> {
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
    let maker_display_name = DisplayName::parse(&r.display_name)
        .map_err(|_| RepoError::Corrupt("Invalid maker display name".into()))?;
    let maker_handle = ProfileHandle::parse(&r.handle)
        .map_err(|_| RepoError::Corrupt("Invalid maker handle".into()))?;
    Ok(PublicOffer {
        offer,
        maker_display_name,
        maker_handle,
    })
}

async fn fetch_cover_images_for_offers(
    pool: &DbPool,
    offer_ids: &[uuid::Uuid],
) -> Result<std::collections::HashMap<uuid::Uuid, haven_domain::image::OfferImage>, RepoError> {
    if offer_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let cover_rows = sqlx::query!(
        r#"
        SELECT id, offer_id, public_id, position, width, height, created_at
        FROM offer_image
        WHERE offer_id = ANY($1) AND position = 1
        "#,
        offer_ids
    )
    .fetch_all(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let mut covers_by_offer = std::collections::HashMap::with_capacity(cover_rows.len());
    for cr in cover_rows {
        let pos_u8 = u8::try_from(cr.position)
            .map_err(|_| RepoError::Corrupt("invalid position in database".into()))?;
        let position = haven_domain::image::ImagePosition::new(pos_u8)
            .map_err(|e| RepoError::Corrupt(e.to_string()))?;
        let public_id = haven_domain::image::ImagePublicId::parse(&cr.public_id)
            .map_err(|e| RepoError::Corrupt(e.to_string()))?;
        let width = u32::try_from(cr.width)
            .map_err(|_| RepoError::Corrupt("invalid width in database".into()))?;
        let height = u32::try_from(cr.height)
            .map_err(|_| RepoError::Corrupt("invalid height in database".into()))?;
        let dimensions = haven_domain::image::ImageDimensions::new(width, height)
            .map_err(|e| RepoError::Corrupt(e.to_string()))?;

        covers_by_offer.insert(
            cr.offer_id,
            haven_domain::image::OfferImage {
                id: haven_domain::image::ImageId::from_uuid(cr.id),
                offer_id: haven_domain::offer::OfferId::from_uuid(cr.offer_id),
                public_id,
                position,
                dimensions,
                created_at: cr.created_at,
            },
        );
    }

    Ok(covers_by_offer)
}

pub(super) async fn list_public(
    pool: &DbPool,
    query: &haven_domain::offer::PublicFeedQuery,
) -> Result<PublicOfferPage, RepoError> {
    let capped_limit = query.limit.clamp(1, 50);
    let fetch_limit = match i64::try_from(capped_limit) {
        Ok(l) => l.saturating_add(1),
        Err(_) => 21,
    };

    let mut sql = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.image_count, o.created_at, o.updated_at,
               u.display_name, u.handle
        FROM offer o
        JOIN "user" u ON o.user_id = u.id
        WHERE o.image_count >= 1
        "#,
    );
    super::criteria::append(&mut sql, &query.criteria);
    if let Some(cursor) = query.cursor.as_ref() {
        sql.push(" AND (o.created_at, o.id) < (")
            .push_bind(cursor.created_at)
            .push(", ")
            .push_bind(cursor.id.as_uuid())
            .push(")");
    }
    sql.push(" ORDER BY o.created_at DESC, o.id DESC LIMIT ")
        .push_bind(fetch_limit);
    let rs = sql
        .build_query_as::<PublicOfferDbRow>()
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?;

    let offer_ids: Vec<uuid::Uuid> = rs.iter().map(|r| r.id).collect();
    let mut covers = fetch_cover_images_for_offers(pool, &offer_ids).await?;

    let mut rows = Vec::with_capacity(rs.len());
    for r in rs {
        let images = covers
            .remove(&r.id)
            .map(|img| vec![img])
            .unwrap_or_default();
        rows.push(map_public_row(r, images)?);
    }

    let has_more = rows.len() > capped_limit;
    if has_more {
        rows.truncate(capped_limit);
    }

    let next_cursor = if has_more {
        rows.last()
            .map(|item| OfferCursor::new(item.offer.created_at, item.offer.id).encode())
    } else {
        None
    };

    Ok(PublicOfferPage {
        items: rows,
        next_cursor,
    })
}

pub(super) async fn list_by_user(
    pool: &DbPool,
    query: UserFeedQuery<'_>,
) -> Result<OfferPage, RepoError> {
    let capped_limit = query.limit.clamp(1, 50);
    let fetch_limit = match i64::try_from(capped_limit) {
        Ok(l) => l.saturating_add(1),
        Err(_) => 21,
    };
    let user_uuid = query.user_id.as_uuid();

    let rs = if let Some(c) = query.cursor {
        let c_time = c.created_at;
        let c_id = c.id.as_uuid();
        sqlx::query_as!(
            UserOfferDbRow,
            r#"
            SELECT id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at
            FROM offer
            WHERE user_id = $1 AND image_count >= 1 AND (created_at, id) < ($2, $3)
            ORDER BY created_at DESC, id DESC
            LIMIT $4
            "#,
            user_uuid,
            c_time,
            c_id,
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?
    } else {
        sqlx::query_as!(
            UserOfferDbRow,
            r#"
            SELECT id, user_id, slug, title, description, price, currency, image_count, created_at, updated_at
            FROM offer
            WHERE user_id = $1 AND image_count >= 1
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#,
            user_uuid,
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?
    };

    let offer_ids: Vec<uuid::Uuid> = rs.iter().map(|r| r.id).collect();
    let mut covers = fetch_cover_images_for_offers(pool, &offer_ids).await?;

    let mut rows = Vec::with_capacity(rs.len());
    for r in rs {
        let images = covers
            .remove(&r.id)
            .map(|img| vec![img])
            .unwrap_or_default();
        rows.push(map_row(
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

    let has_more = rows.len() > capped_limit;
    if has_more {
        rows.truncate(capped_limit);
    }

    let next_cursor = if has_more {
        rows.last()
            .map(|item| OfferCursor::new(item.created_at, item.id).encode())
    } else {
        None
    };

    Ok(OfferPage {
        items: rows,
        next_cursor,
    })
}
