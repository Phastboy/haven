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
    let activity_at = match query.scope() {
        PersonalScope::Saved(_) => "x.created_at",
        PersonalScope::Inquired(_) => "x.last_inquiry_at",
    };
    let mut sql = QueryBuilder::<Postgres>::new(
        "WITH matching AS MATERIALIZED (SELECT o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.category, o.image_count, o.created_at, o.updated_at, ",
    );
    sql.push(activity_at)
        .push(" AS activity_at, ")
        .push(key)
        .push(" AS activity_id FROM offer o ");
    match query.scope() {
        PersonalScope::Saved(person) => {
            sql.push("JOIN saved_offer x ON x.offer_id = o.id WHERE x.user_id = ")
                .push_bind(person.as_uuid());
        }
        PersonalScope::Inquired(person) => {
            sql.push(
                "JOIN person_inquired_offer_activity x ON x.offer_id = o.id WHERE x.person_id = ",
            )
            .push_bind(person.as_uuid());
        }
    }
    sql.push(" AND o.image_count >= 1");
    super::criteria::append(&mut sql, query.criteria());
    if let Some(position) = query.position() {
        sql.push(" AND (")
            .push(activity_at)
            .push(", ")
            .push(key)
            .push(") < (")
            .push_bind(position.at)
            .push(", ")
            .push_bind(position.id)
            .push(")");
    }
    // Bound membership/facets before loading maker details, so planner row
    // estimates cannot turn the maker join into repeated full-table scans.
    sql.push(" ORDER BY ")
        .push(activity_at)
        .push(" DESC, ")
        .push(key)
        .push(" DESC LIMIT 21) SELECT matching.*, u.display_name, u.handle FROM matching JOIN \"user\" u ON u.id = matching.user_id ORDER BY activity_at DESC, activity_id DESC");
    let mut rows = sql
        .build_query_as::<ActivityRow>()
        // Facet selectivity varies sharply; a cached generic plan can scan the
        // entire matching history before LIMIT instead of streaming the keyset.
        .persistent(false)
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
