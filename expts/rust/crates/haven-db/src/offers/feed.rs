//! Public offer feed keyset pagination query.

use crate::DbPool;
use haven_domain::offer::{Offer, OfferCursor, OfferPage};
use haven_domain::ports::RepoError;

use super::rows::map_row;

pub(super) async fn list_public(
    pool: &DbPool,
    cursor: Option<&OfferCursor>,
    limit: usize,
) -> Result<OfferPage, RepoError> {
    let capped_limit = limit.clamp(1, 50);
    let fetch_limit = match i64::try_from(capped_limit) {
        Ok(l) => l.saturating_add(1),
        Err(_) => 21,
    };

    let mut rows: Vec<Offer> = if let Some(c) = cursor {
        let c_time = c.created_at;
        let c_id = c.id.as_uuid();
        let rs = sqlx::query!(
            r#"
                SELECT id, user_id, slug, title, description, price, currency, created_at, updated_at
                FROM offer
                WHERE (created_at, id) < ($1, $2)
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#,
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
                ORDER BY created_at DESC, id DESC
                LIMIT $1
                "#,
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
