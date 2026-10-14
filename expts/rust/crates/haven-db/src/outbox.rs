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
    /// Number of attempts made, including the currently claimed attempt.
    pub attempts: i32,
}

/// A dead-lettered purge item that reached terminal failure (5 or more failed attempts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterPurgeItem {
    /// Unique ID of the outbox record.
    pub id: Uuid,
    /// Storage public ID to delete.
    pub public_id: String,
    /// Total number of attempts made.
    pub attempts: i32,
    /// Error message recorded on the last failure.
    pub last_error: Option<String>,
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
        RETURNING image_purge_outbox.id, image_purge_outbox.public_id, image_purge_outbox.attempts
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
            attempts: r.attempts,
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

/// Lists dead-lettered purge records that have exhausted all retry attempts (attempts >= 5).
pub async fn list_failed_purges(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<DeadLetterPurgeItem>, RepoError> {
    let records = sqlx::query!(
        r#"
        SELECT id, public_id, attempts, last_error
        FROM image_purge_outbox
        WHERE attempts >= 5
        ORDER BY created_at ASC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(records
        .into_iter()
        .map(|r| DeadLetterPurgeItem {
            id: r.id,
            public_id: r.public_id,
            attempts: r.attempts,
            last_error: r.last_error,
        })
        .collect())
}

/// Requeues a single failed or dead-lettered purge record by resetting its attempt counter.
///
/// Returns `true` if a record was found and reset, `false` otherwise.
pub async fn requeue_failed_purge(pool: &DbPool, id: Uuid) -> Result<bool, RepoError> {
    let result = sqlx::query!(
        r#"
        UPDATE image_purge_outbox
        SET attempts = 0, last_attempt_at = NULL, last_error = NULL
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(result.rows_affected() > 0)
}

/// Requeues all dead-lettered purge records (records with 5 or more failed attempts).
///
/// Returns the number of records requeued for reprocessing.
pub async fn requeue_all_failed_purges(pool: &DbPool) -> Result<u64, RepoError> {
    let result = sqlx::query!(
        r#"
        UPDATE image_purge_outbox
        SET attempts = 0, last_attempt_at = NULL, last_error = NULL
        WHERE attempts >= 5
        "#
    )
    .execute(pool)
    .await
    .map_err(map_sqlx_err)?;

    Ok(result.rows_affected())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "test assertions"
)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_outbox_claim_record_failure_and_dead_letter_requeue(pool: PgPool) {
        let public_id = "test/purge/dlq_asset".to_string();

        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO image_purge_outbox (public_id) VALUES ($1) RETURNING id",
        )
        .bind(&public_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        // Claim 1: attempts becomes 1
        let claimed = claim_pending_image_purges(&pool, 10).await.unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].id, id);
        assert_eq!(claimed[0].attempts, 1);

        // Immediate subsequent claim: skipped because of exponential backoff window
        let immediate_claimed = claim_pending_image_purges(&pool, 10).await.unwrap();
        assert_eq!(immediate_claimed.len(), 0);

        // Fast-forward attempts to 5 (terminal failure)
        sqlx::query(
            "UPDATE image_purge_outbox SET attempts = 5, last_attempt_at = NOW() - INTERVAL '1 day', last_error = 'Storage connection timeout' WHERE id = $1",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

        // Terminal item is not claimed by worker query
        let terminal_claimed = claim_pending_image_purges(&pool, 10).await.unwrap();
        assert_eq!(terminal_claimed.len(), 0);

        // Shows in dead-letter list
        let dead_letters = list_failed_purges(&pool, 10).await.unwrap();
        assert_eq!(dead_letters.len(), 1);
        assert_eq!(dead_letters[0].id, id);
        assert_eq!(dead_letters[0].attempts, 5);
        assert_eq!(
            dead_letters[0].last_error.as_deref(),
            Some("Storage connection timeout")
        );

        // Requeue by ID
        let requeued = requeue_failed_purge(&pool, id).await.unwrap();
        assert!(requeued);

        // Can now be claimed again
        let reclaimed = claim_pending_image_purges(&pool, 10).await.unwrap();
        assert_eq!(reclaimed.len(), 1);
        assert_eq!(reclaimed[0].id, id);
        assert_eq!(reclaimed[0].attempts, 1);

        // Fast-forward to 5 again and test batch requeue
        sqlx::query("UPDATE image_purge_outbox SET attempts = 5 WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();

        let count = requeue_all_failed_purges(&pool).await.unwrap();
        assert_eq!(count, 1);

        let dead_letters_after = list_failed_purges(&pool, 10).await.unwrap();
        assert_eq!(dead_letters_after.len(), 0);
    }
}
