//! Indexed unread checks and conversation-serialized read transitions.

use chrono::Utc;
use haven_domain::conversation::ConversationId;
use haven_domain::ports::RepoError;
use haven_domain::user::UserId;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::DbPool;

pub(super) async fn lock_participant(
    tx: &mut Transaction<'_, Postgres>,
    conversation_id: ConversationId,
    person: UserId,
) -> Result<(), RepoError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM conversation WHERE id = $1 AND (user_low = $2 OR user_high = $2) FOR UPDATE",
    )
    .bind(conversation_id.as_uuid())
    .bind(person.as_uuid())
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::map_sqlx_err)?;
    if id.is_none() {
        return Err(RepoError::NotFound {
            entity: "conversation",
            id: conversation_id.to_string(),
        });
    }
    Ok(())
}

pub async fn mark_read(
    pool: &DbPool,
    conversation_id: ConversationId,
    user_id: UserId,
) -> Result<(), RepoError> {
    let c_id = conversation_id.as_uuid();
    let u_id = user_id.as_uuid();
    let mut tx = pool.begin().await.map_err(crate::map_sqlx_err)?;
    lock_participant(&mut tx, conversation_id, user_id).await?;
    let now = Utc::now();

    sqlx::query!(
        r#"
        INSERT INTO conversation_participant (conversation_id, user_id, last_read_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (conversation_id, user_id)
        DO UPDATE SET last_read_at = EXCLUDED.last_read_at
        "#,
        c_id,
        u_id,
        now
    )
    .execute(&mut *tx)
    .await
    .map_err(crate::map_sqlx_err)?;

    tx.commit().await.map_err(crate::map_sqlx_err)?;
    Ok(())
}

pub async fn unread_exists(pool: &DbPool, user_id: UserId) -> Result<bool, RepoError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM conversation_participant WHERE user_id = $1 AND is_unread)",
    )
    .bind(user_id.as_uuid())
    .fetch_one(pool)
    .await
    .map_err(crate::map_sqlx_err)
}
