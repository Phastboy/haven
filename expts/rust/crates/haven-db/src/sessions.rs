use crate::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use haven_domain::account::AccountId;
use haven_domain::ports::{RepoError, SessionRepository};
use haven_domain::session::{HashedToken, Session, SessionId};
use std::net::IpAddr;
use uuid::Uuid;

pub(crate) struct PostgresSessionRepository {
    pool: DbPool,
}

impl PostgresSessionRepository {
    pub(crate) fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Internal database row mapping from individual table columns"
    )]
    fn map_row(
        id: Uuid,
        account_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Session {
        Session {
            id: SessionId::from_uuid(id),
            account_id: AccountId::from_uuid(account_id),
            token_hash: HashedToken::from_hex(token_hash),
            expires_at,
            created_at,
            ip_address,
            user_agent,
        }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn find_by_token_hash(
        &self,
        token_hash: &HashedToken,
    ) -> Result<Option<Session>, RepoError> {
        let hash_str = token_hash.as_str();
        let row = sqlx::query!(
            r#"
            SELECT id, account_id, token_hash, expires_at, created_at, ip_address, user_agent
            FROM session
            WHERE token_hash = $1 AND expires_at > now()
            "#,
            hash_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Ok(row.map(|r| {
            Self::map_row(
                r.id,
                r.account_id,
                r.token_hash,
                r.expires_at,
                r.created_at,
                r.ip_address,
                r.user_agent,
            )
        }))
    }

    async fn create(
        &self,
        account_id: AccountId,
        token_hash: &HashedToken,
        expires_at: DateTime<Utc>,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<Session, RepoError> {
        let id_uuid = account_id.as_uuid();
        let hash_str = token_hash.as_str();
        let ip_str = ip_address.map(|ip| ip.to_string());

        let row = sqlx::query!(
            r#"
            INSERT INTO session (account_id, token_hash, expires_at, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, account_id, token_hash, expires_at, created_at, ip_address, user_agent
            "#,
            id_uuid,
            hash_str,
            expires_at,
            ip_str,
            user_agent
        )
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Ok(Self::map_row(
            row.id,
            row.account_id,
            row.token_hash,
            row.expires_at,
            row.created_at,
            row.ip_address,
            row.user_agent,
        ))
    }

    async fn delete(&self, session_id: SessionId) -> Result<(), RepoError> {
        let id_uuid = session_id.as_uuid();
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM session
            WHERE id = $1
            "#,
            id_uuid
        )
        .execute(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?
        .rows_affected();

        if rows_affected == 0 {
            Err(RepoError::NotFound {
                entity: "session",
                id: session_id.to_string(),
            })
        } else {
            Ok(())
        }
    }
}
