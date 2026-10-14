//! Transactional outbox persistence for asynchronous image purges.

use crate::{DbPool, map_sqlx_err};
use haven_domain::ports::RepoError;
use uuid::Uuid;

/// A pending asset purge queued by database engine triggers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxPurgeItem {
    /// Unique ID of the outbox record.
    pub id: Uuid,
    /// Storage public ID to delete.
    pub public_id: String,
}

/// Atomically claims up to `limit` pending image purges using `SKIP LOCKED`.
///
/// Increments attempts and updates `last_attempt_at` to prevent competing workers
/// from processing the same records during their backoff window.
pub async fn claim_pending_image_purges(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<OutboxPurgeItem>, RepoError> {
    let records = sqlx::query!(
        r#"
        WITH claimed AS (
            SELECT id
            FROM image_purge_outbox
            WHERE attempts < 5
              AND (last_attempt_at IS NULL OR last_attempt_at < NOW() - (INTERVAL '10 seconds' * POWER(2, LEAST(attempts, 4))))
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE image_purge_outbox
        SET attempts = attempts + 1, last_attempt_at = NOW()
        FROM claimed
        WHERE image_purge_outbox.id = claimed.id
        RETURNING image_purge_outbox.id, image_purge_outbox.public_id
        "#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(records
        .into_iter()
        .map(|r| OutboxPurgeItem {
            id: r.id,
            public_id: r.public_id,
        })
        .collect())
}

/// Marks an image purge as successfully completed by deleting it from the outbox.
pub async fn complete_image_purge(pool: &DbPool, id: Uuid) -> Result<(), RepoError> {
    sqlx::query!(
        r#"
        DELETE FROM image_purge_outbox
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(())
}

/// Records a failed purge attempt with its error message.
pub async fn record_purge_failure(
    pool: &DbPool,
    id: Uuid,
    error_msg: &str,
) -> Result<(), RepoError> {
    sqlx::query!(
        r#"
        UPDATE image_purge_outbox
        SET last_error = $2
        WHERE id = $1
        "#,
        id,
        error_msg
    )
    .execute(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(())
}
