use async_trait::async_trait;
use chrono::{DateTime, Utc};
use haven_domain::account::Email;
use haven_domain::magic_link::MagicLink;
use haven_domain::ports::{MagicLinkRepository, RepoError};
use haven_domain::session::HashedToken;
use crate::DbPool;
use uuid::Uuid;

pub struct PostgresMagicLinkRepository {
    pub pool: DbPool,
}

impl PostgresMagicLinkRepository {
    fn map_row(
        id: Uuid,
        email: &str,
        token_hash: String,
        expires_at: DateTime<Utc>,
        used_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Result<MagicLink, RepoError> {
        let e = Email::parse(email).map_err(|_| RepoError::Corrupt("Invalid email".into()))?;
        Ok(MagicLink {
            id,
            email: e,
            token_hash: HashedToken::from_hex(token_hash),
            expires_at,
            used_at,
            created_at,
        })
    }
}

#[async_trait]
impl MagicLinkRepository for PostgresMagicLinkRepository {
    /// Finds an unused, unexpired magic link by its token hash.
    async fn find_by_token_hash(
        &self,
        token_hash: &HashedToken,
    ) -> Result<Option<MagicLink>, RepoError> {
        let hash_str = token_hash.as_str();
        let row = sqlx::query!(
            r#"
            SELECT id, email, token_hash, expires_at, used_at, created_at
            FROM magic_link
            WHERE token_hash = $1 AND expires_at > now()
            "#,
            hash_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(
                r.id,
                &r.email,
                r.token_hash,
                r.expires_at,
                r.used_at,
                r.created_at,
            )
        })
        .transpose()
    }

    /// Creates a new magic link and invalidates any previous unused links for the same email
    /// within a single transaction.
    async fn create(
        &self,
        email: &Email,
        token_hash: &HashedToken,
        expires_at: DateTime<Utc>,
    ) -> Result<MagicLink, RepoError> {
        let email_str = email.as_str();
        let hash_str = token_hash.as_str();

        let mut tx = self.pool.begin().await.map_err(crate::map_sqlx_err)?;

        // Invalidate previous magic links for this email
        sqlx::query!(
            "UPDATE magic_link SET used_at = now() WHERE email = $1 AND used_at IS NULL",
            email_str
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

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
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

        tx.commit().await.map_err(crate::map_sqlx_err)?;

        Self::map_row(
            row.id,
            &row.email,
            row.token_hash,
            row.expires_at,
            row.used_at,
            row.created_at,
        )
    }

    /// Marks a magic link as used. Requires the link to be currently unused and not expired.
    async fn mark_used(&self, magic_link_id: Uuid) -> Result<(), RepoError> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE magic_link
            SET used_at = now()
            WHERE id = $1 AND used_at IS NULL AND expires_at > now()
            "#,
            magic_link_id
        )
        .execute(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?
        .rows_affected();

        if rows_affected == 0 {
            Err(RepoError::NotFound)
        } else {
            Ok(())
        }
    }
}
