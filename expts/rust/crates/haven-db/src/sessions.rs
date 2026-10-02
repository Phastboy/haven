use chrono::{DateTime, Utc};
use haven_domain::account::AccountId;
use haven_domain::session::{HashedToken, Session, SessionId};

use crate::DbPool;

pub async fn create(
    pool: &DbPool,
    account_id: AccountId,
    token_hash: &HashedToken,
    expires_at: DateTime<Utc>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> sqlx::Result<Session> {
    let id_uuid = account_id.as_uuid();
    let hash_str = token_hash.as_str();
    let row = sqlx::query!(
        r#"
        INSERT INTO session (account_id, token_hash, expires_at, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, account_id, token_hash, expires_at, created_at, ip_address, user_agent
        "#,
        id_uuid,
        hash_str,
        expires_at,
        ip_address,
        user_agent
    )
    .fetch_one(pool)
    .await?;

    Ok(Session {
        id: SessionId(row.id),
        account_id: AccountId(row.account_id),
        token_hash: HashedToken::from_hex(row.token_hash),
        expires_at: row.expires_at,
        created_at: row.created_at,
        ip_address: row.ip_address,
        user_agent: row.user_agent,
    })
}

pub async fn find_by_token_hash(
    pool: &DbPool,
    token_hash: &HashedToken,
) -> sqlx::Result<Option<Session>> {
    let hash_str = token_hash.as_str();
    let row = sqlx::query!(
        r#"
        SELECT id, account_id, token_hash, expires_at, created_at, ip_address, user_agent
        FROM session
        WHERE token_hash = $1
        "#,
        hash_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Session {
        id: SessionId(r.id),
        account_id: AccountId(r.account_id),
        token_hash: HashedToken::from_hex(r.token_hash),
        expires_at: r.expires_at,
        created_at: r.created_at,
        ip_address: r.ip_address,
        user_agent: r.user_agent,
    }))
}

pub async fn delete(pool: &DbPool, session_id: SessionId) -> sqlx::Result<()> {
    let id_uuid = session_id.0;
    sqlx::query!(
        r#"
        DELETE FROM session
        WHERE id = $1
        "#,
        id_uuid
    )
    .execute(pool)
    .await?;
    Ok(())
}
