//! Public offer feed keyset pagination query.

use crate::DbPool;
use haven_domain::offer::{Offer, OfferCursor, OfferPage, PublicOffer, PublicOfferPage, UserId};
use haven_domain::ports::RepoError;
use haven_domain::user::{DisplayName, ProfileHandle};

use super::rows::map_row;

/// Query parameters for fetching a user's offer feed.
pub(super) struct UserFeedQuery<'a> {
    pub user_id: UserId,
    pub cursor: Option<&'a OfferCursor>,
    pub limit: usize,
}

struct PublicOfferDbRow {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub display_name: String,
    pub handle: String,
}

fn map_public_row(r: PublicOfferDbRow) -> Result<PublicOffer, RepoError> {
    let offer = map_row(
        r.id,
        r.user_id,
        r.slug,
        r.title,
        r.description,
        r.price,
        &r.currency,
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

pub(super) async fn list_public(
    pool: &DbPool,
    cursor: Option<&OfferCursor>,
    limit: usize,
) -> Result<PublicOfferPage, RepoError> {
    let capped_limit = limit.clamp(1, 50);
    let fetch_limit = match i64::try_from(capped_limit) {
        Ok(l) => l.saturating_add(1),
        Err(_) => 21,
    };

    let mut rows: Vec<PublicOffer> = if let Some(c) = cursor {
        let rs = sqlx::query_as!(
            PublicOfferDbRow,
            r#"
            SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.created_at, o.updated_at,
                   u.display_name, u.handle
            FROM offer o
            JOIN "user" u ON o.user_id = u.id
            WHERE (o.created_at, o.id) < ($1, $2)
            ORDER BY o.created_at DESC, o.id DESC
            LIMIT $3
            "#,
            c.created_at,
            c.id.as_uuid(),
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        let mut items = Vec::with_capacity(rs.len());
        for r in rs {
            items.push(map_public_row(r)?);
        }
        items
    } else {
        let rs = sqlx::query_as!(
            PublicOfferDbRow,
            r#"
            SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.created_at, o.updated_at,
                   u.display_name, u.handle
            FROM offer o
            JOIN "user" u ON o.user_id = u.id
            ORDER BY o.created_at DESC, o.id DESC
            LIMIT $1
            "#,
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        let mut items = Vec::with_capacity(rs.len());
        for r in rs {
            items.push(map_public_row(r)?);
        }
        items
    };

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

    let mut rows: Vec<Offer> = if let Some(c) = query.cursor {
        let c_time = c.created_at;
        let c_id = c.id.as_uuid();
        let rs = sqlx::query!(
            r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE user_id = $1 AND (created_at, id) < ($2, $3)
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
        .map_err(crate::map_sqlx_err)?;

        let mut items = Vec::with_capacity(rs.len());
        for r in rs {
            items.push(map_row(
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
        items
    } else {
        let rs = sqlx::query!(
            r#"
            SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
            FROM offer
            WHERE user_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2
            "#,
            user_uuid,
            fetch_limit
        )
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        let mut items = Vec::with_capacity(rs.len());
        for r in rs {
            items.push(map_row(
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
        items
    };

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
