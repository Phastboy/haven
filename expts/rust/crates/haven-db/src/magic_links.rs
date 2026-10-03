use chrono::{DateTime, Utc};
use haven_domain::account::Email;
use haven_domain::magic_link::MagicLink;
use haven_domain::session::HashedToken;
use uuid::Uuid;

use crate::DbPool;

pub async fn create(
    pool: &DbPool,
    email: &Email,
    token_hash: &HashedToken,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<MagicLink> {
    let email_str = email.as_str();
    let hash_str = token_hash.as_str();
    
    // Invalidate previous magic links for this email
    sqlx::query!(
        "UPDATE magic_link SET used_at = now() WHERE email = $1 AND used_at IS NULL",
        email_str
    ).execute(pool).await?;

    let row = sqlx::query!(
        r#"
        INSERT INTO magic_link (email, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, email, token_hash, expires_at, used_at, created_at
        "#,
        email_str,
        hash_str,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(MagicLink {
        id: row.id,
        email: Email::parse(&row.email).expect("db email must be valid"),
        token_hash: HashedToken::from_hex(row.token_hash),
        expires_at: row.expires_at,
        used_at: row.used_at,
        created_at: row.created_at,
    })
}

pub async fn find_by_token_hash(
    pool: &DbPool,
    token_hash: &HashedToken,
) -> sqlx::Result<Option<MagicLink>> {
    let hash_str = token_hash.as_str();
    let row = sqlx::query!(
        r#"
        SELECT id, email, token_hash, expires_at, used_at, created_at
        FROM magic_link
        WHERE token_hash = $1 AND expires_at > now()
        "#,
        hash_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| MagicLink {
        id: r.id,
        email: Email::parse(&r.email).expect("db email must be valid"),
        token_hash: HashedToken::from_hex(r.token_hash),
        expires_at: r.expires_at,
        used_at: r.used_at,
        created_at: r.created_at,
    }))
}

pub async fn consume(pool: &DbPool, magic_link_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        UPDATE magic_link
        SET used_at = now()
        WHERE id = $1
        "#,
        magic_link_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
