//! Personal membership and activity ordering, intersected with facets before LIMIT.

use super::feed::{PublicOfferDbRow, fetch_cover_images_for_offers, map_public_row};
use haven_domain::offer::{
    ActivityPosition, PersonalCursor, PersonalFeedQuery, PersonalScope, PublicOfferPage,
};
use haven_domain::ports::RepoError;
use sqlx::{Postgres, QueryBuilder};

#[derive(sqlx::FromRow)]
struct ActivityRow {
    #[sqlx(flatten)]
    offer: PublicOfferDbRow,
    activity_at: chrono::DateTime<chrono::Utc>,
    activity_id: uuid::Uuid,
}

pub(super) async fn list(
    pool: &crate::DbPool,
    query: &PersonalFeedQuery,
) -> Result<PublicOfferPage, RepoError> {
    let key = match query.scope() {
        PersonalScope::Saved(_) => "x.id",
        PersonalScope::Inquired(_) => "x.offer_id",
    };
    let mut sql = QueryBuilder::<Postgres>::new(
        "SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.category, o.image_count, o.created_at, o.updated_at, u.display_name, u.handle, x.created_at AS activity_at, ",
    );
    sql.push(key)
        .push(" AS activity_id FROM offer o JOIN \"user\" u ON u.id = o.user_id ");
    match query.scope() {
        PersonalScope::Saved(person) => {
            sql.push("JOIN saved_offer x ON x.offer_id = o.id WHERE x.user_id = ")
                .push_bind(person.as_uuid());
        }
        PersonalScope::Inquired(person) => {
            sql.push("JOIN message x ON x.offer_id = o.id JOIN conversation c ON c.id = x.conversation_id WHERE x.sender_id = ").push_bind(person.as_uuid());
            sql.push(" AND x.offer_id IS NOT NULL AND o.user_id <> x.sender_id AND c.user_low = LEAST(x.sender_id, o.user_id) AND c.user_high = GREATEST(x.sender_id, o.user_id) AND x.id = (SELECT newer.id FROM message newer WHERE newer.sender_id = x.sender_id AND newer.offer_id = x.offer_id AND newer.offer_id IS NOT NULL AND newer.conversation_id = x.conversation_id ORDER BY newer.created_at DESC, newer.id DESC LIMIT 1)");
        }
    }
    sql.push(" AND o.image_count >= 1");
    super::criteria::append(&mut sql, query.criteria());
    if let Some(position) = query.position() {
        sql.push(" AND (x.created_at, ")
            .push(key)
            .push(") < (")
            .push_bind(position.at)
            .push(", ")
            .push_bind(position.id)
            .push(")");
    }
    sql.push(" ORDER BY x.created_at DESC, ")
        .push(key)
        .push(" DESC LIMIT 21");
    let mut rows = sql
        .build_query_as::<ActivityRow>()
        .fetch_all(pool)
        .await
        .map_err(crate::map_sqlx_err)?;
    let has_more = rows.len() > 20;
    rows.truncate(20);
    let next_cursor = if has_more {
        rows.last().map(|row| {
            PersonalCursor::new(
                query.scope(),
                query.criteria(),
                ActivityPosition {
                    at: row.activity_at,
                    id: row.activity_id,
                },
            )
            .encode()
        })
    } else {
        None
    };
    let ids: Vec<_> = rows.iter().map(|row| row.offer.id).collect();
    let mut covers = fetch_cover_images_for_offers(pool, &ids).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let images = covers
            .remove(&row.offer.id)
            .map(|image| vec![image])
            .unwrap_or_default();
        items.push(map_public_row(row.offer, images)?);
    }
    Ok(PublicOfferPage { items, next_cursor })
}
