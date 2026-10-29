//! Qualifying histories plus replies and invalid tags that must never become membership.
use super::TruthRow;
use chrono::{DateTime, Utc};
use haven_domain::offer::ActivityPosition;
use haven_domain::user::UserId;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct SavedRow {
    offer_id: Uuid,
    id: Uuid,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Copy)]
pub(super) struct HistorySeed {
    pub(super) person: UserId,
    pub(super) count: usize,
    pub(super) reserved: UserId,
}

pub(super) async fn seed(
    pool: &PgPool,
    rows: &[TruthRow],
    selection: HistorySeed,
) -> Result<
    (
        HashMap<Uuid, ActivityPosition>,
        HashMap<Uuid, ActivityPosition>,
    ),
    Box<dyn std::error::Error>,
> {
    let HistorySeed {
        person,
        count,
        reserved,
    } = selection;
    let ids: Vec<_> = rows
        .iter()
        .filter(|row| row.user_id != person.as_uuid() && row.user_id != reserved.as_uuid())
        .take(count)
        .map(|row| row.id)
        .collect();
    sqlx::query("INSERT INTO saved_offer (user_id,offer_id,created_at) SELECT $1,u.id,'2026-02-01'::timestamptz + ((u.n-1)/4)::int * interval '1 second' FROM unnest($2::uuid[]) WITH ORDINALITY u(id,n)")
        .bind(person.as_uuid()).bind(&ids).execute(pool).await?;
    let saved = sqlx::query_as::<_, SavedRow>(
        "SELECT offer_id,id,created_at FROM saved_offer WHERE user_id=$1",
    )
    .bind(person.as_uuid())
    .fetch_all(pool)
    .await?;
    let saved = saved
        .into_iter()
        .map(|row| {
            (
                row.offer_id,
                ActivityPosition {
                    at: row.created_at,
                    id: row.id,
                },
            )
        })
        .collect();
    sqlx::query("INSERT INTO conversation (user_low,user_high) SELECT DISTINCT LEAST($1::uuid,o.user_id),GREATEST($1::uuid,o.user_id) FROM offer o WHERE o.id=ANY($2::uuid[]) ON CONFLICT (user_low,user_high) DO NOTHING")
        .bind(person.as_uuid()).bind(&ids).execute(pool).await?;
    sqlx::query("INSERT INTO message (conversation_id,sender_id,offer_id,body,created_at) SELECT c.id,$1,o.id,'Synthetic qualifying inquiry','2026-02-01'::timestamptz + ((u.n-1)/4)::int * interval '1 second' + r * interval '0.001 second' FROM unnest($2::uuid[]) WITH ORDINALITY u(id,n) JOIN offer o ON o.id=u.id JOIN conversation c ON c.user_low=LEAST($1::uuid,o.user_id) AND c.user_high=GREATEST($1::uuid,o.user_id) CROSS JOIN generate_series(1,3) r")
        .bind(person.as_uuid()).bind(&ids).execute(pool).await?;
    // Owner replies do not move Inquired, including replies about multiple offers.
    sqlx::query("INSERT INTO message (conversation_id,sender_id,offer_id,body,created_at) SELECT c.id,o.user_id,o.id,'Synthetic owner reply','2026-03-01'::timestamptz FROM offer o JOIN conversation c ON c.user_low=LEAST($1::uuid,o.user_id) AND c.user_high=GREATEST($1::uuid,o.user_id) WHERE o.id=ANY($2::uuid[])")
        .bind(person.as_uuid()).bind(&ids).execute(pool).await?;
    sqlx::query("INSERT INTO conversation_participant (conversation_id,user_id,last_read_at) SELECT c.id,$1,'2026-04-01'::timestamptz FROM conversation c WHERE c.user_low=$1 OR c.user_high=$1 ON CONFLICT (conversation_id,user_id) DO UPDATE SET last_read_at=EXCLUDED.last_read_at")
        .bind(person.as_uuid()).execute(pool).await?;
    sqlx::query("UPDATE conversation c SET last_message_at=(SELECT MAX(created_at) FROM message m WHERE m.conversation_id=c.id) WHERE (c.user_low=$1 OR c.user_high=$1) AND EXISTS(SELECT 1 FROM message m WHERE m.conversation_id=c.id)")
        .bind(person.as_uuid()).execute(pool).await?;
    let base = DateTime::parse_from_rfc3339("2026-02-01T00:00:00.003Z")?.with_timezone(&Utc);
    let inquired = ids
        .iter()
        .enumerate()
        .map(|(n, id)| {
            (
                *id,
                ActivityPosition {
                    at: base + chrono::Duration::seconds(i64::try_from(n / 4).unwrap()),
                    id: *id,
                },
            )
        })
        .collect();
    Ok((saved, inquired))
}
