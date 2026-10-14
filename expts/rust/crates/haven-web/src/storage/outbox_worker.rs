//! Asynchronous background worker for processing transactional image purge outbox items.

use haven_db::DbPool;
use haven_domain::ports::ImageStorage;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for the outbox purge background worker.
#[derive(Debug, Clone)]
pub struct OutboxWorkerConfig {
    /// Batch size of outbox records to process in one cycle.
    pub batch_size: i64,
    /// Delay between polling cycles when idle.
    pub poll_interval: Duration,
}

impl Default for OutboxWorkerConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            poll_interval: Duration::from_secs(5),
        }
    }
}

/// Spawns the background image purge outbox worker task.
pub fn spawn_outbox_worker(
    pool: DbPool,
    storage: Arc<dyn ImageStorage>,
    config: OutboxWorkerConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        run_worker(pool, storage, config).await;
    })
}

/// Runs a single iteration of processing pending image purges.
///
/// Returns the number of items processed in this pass.
pub async fn process_pending_purges(
    pool: &DbPool,
    storage: &dyn ImageStorage,
    batch_size: i64,
) -> usize {
    match haven_db::outbox::claim_pending_image_purges(pool, batch_size).await {
        Ok(items) => {
            let count = items.len();
            for item in items {
                match storage.delete(&item.public_id).await {
                    Ok(()) => {
                        if let Err(e) = haven_db::outbox::complete_image_purge(pool, item.id).await
                        {
                            writeln!(
                                std::io::stderr(),
                                "[outbox_worker] failed to mark purge complete for {}: {e}",
                                item.id
                            )
                            .unwrap_or(());
                        }
                    }
                    Err(e) => {
                        writeln!(
                            std::io::stderr(),
                            "[outbox_worker] failed to delete asset {}: {e}",
                            item.public_id
                        )
                        .unwrap_or(());
                        if let Err(db_err) =
                            haven_db::outbox::record_purge_failure(pool, item.id, &e.to_string())
                                .await
                        {
                            writeln!(
                                std::io::stderr(),
                                "[outbox_worker] failed to record purge failure for {}: {db_err}",
                                item.id
                            )
                            .unwrap_or(());
                        }
                    }
                }
            }
            count
        }
        Err(e) => {
            writeln!(
                std::io::stderr(),
                "[outbox_worker] error claiming purges: {e}"
            )
            .unwrap_or(());
            0
        }
    }
}

async fn run_worker(pool: DbPool, storage: Arc<dyn ImageStorage>, config: OutboxWorkerConfig) {
    loop {
        let processed = process_pending_purges(&pool, storage.as_ref(), config.batch_size).await;
        if processed == 0 {
            sleep(config.poll_interval).await;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::storage::MockImageStorage;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_outbox_worker_processes_and_cleans_up(pool: PgPool) {
        let storage = Arc::new(MockImageStorage::new());
        let public_id = "test/purge/asset_123".to_string();

        sqlx::query("INSERT INTO image_purge_outbox (public_id) VALUES ($1)")
            .bind(&public_id)
            .execute(&pool)
            .await
            .unwrap();

        let processed = process_pending_purges(&pool, storage.as_ref(), 10).await;
        assert_eq!(processed, 1);

        let deleted = storage.deleted_ids();
        assert_eq!(deleted, vec![public_id.clone()]);

        let remaining = sqlx::query("SELECT id FROM image_purge_outbox WHERE public_id = $1")
            .bind(&public_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(remaining.is_none());
    }
}
