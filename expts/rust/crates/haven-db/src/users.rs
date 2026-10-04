use async_trait::async_trait;
use haven_domain::account::AccountId;
use haven_domain::ports::{RepoError, UserRepository};
use haven_domain::session::HashedToken;
use haven_domain::user::User;
use haven_domain::offer::UserId;
use crate::DbPool;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pub pool: DbPool,
}

impl PostgresUserRepository {
    fn map_row(
        id: Uuid,
        account_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> User {
        User {
            id: UserId(id),
            account_id: AccountId(account_id),
            created_at,
            updated_at,
        }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_account(&self, account_id: AccountId) -> Result<Option<User>, RepoError> {
        let id_uuid = account_id.as_uuid();
        let row = sqlx::query!(
            r#"
            SELECT id, account_id, created_at, updated_at
            FROM "user"
            WHERE account_id = $1
            "#,
            id_uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Ok(row.map(|r| {
            Self::map_row(
                r.id,
                r.account_id,
                r.created_at,
                r.updated_at,
            )
        }))
    }

    async fn create(&self, account_id: AccountId) -> Result<User, RepoError> {
        let id_uuid = account_id.as_uuid();
        let row = sqlx::query!(
            r#"
            INSERT INTO "user" (account_id)
            VALUES ($1)
            RETURNING id, account_id, created_at, updated_at
            "#,
            id_uuid
        )
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Ok(Self::map_row(
            row.id,
            row.account_id,
            row.created_at,
            row.updated_at,
        ))
    }

    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError> {
        let hash_str = token_hash.as_str();
        let row = sqlx::query!(
            r#"
            SELECT u.id, u.account_id, u.created_at, u.updated_at
            FROM session s
            JOIN "user" u ON s.account_id = u.account_id
            WHERE s.token_hash = $1 AND s.expires_at > now()
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
                r.created_at,
                r.updated_at,
            )
        }))
    }
}
