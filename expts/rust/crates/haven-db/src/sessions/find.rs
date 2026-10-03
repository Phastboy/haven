use haven_domain::account::AccountId;
use haven_domain::session::{HashedToken, Session, SessionId};
use crate::DbPool;

pub async fn find_by_token_hash(
    pool: &DbPool,
    token_hash: &HashedToken,
) -> sqlx::Result<Option<Session>> {
    let hash_str = token_hash.as_str();
    let row = sqlx::query!(
        r#"
        SELECT id, account_id, token_hash, expires_at, created_at, ip_address, user_agent
        FROM session
        WHERE token_hash = $1 AND expires_at > now()
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
