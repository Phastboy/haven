use async_trait::async_trait;
use haven_domain::account::{Account, AccountId, Email};
use haven_domain::ports::{AccountRepository, RepoError};
use crate::DbPool;
use uuid::Uuid;

pub struct PostgresAccountRepository {
    pub pool: DbPool,
}

impl PostgresAccountRepository {
    fn map_row(
        id: Uuid,
        email: &str,
        email_verified: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Account, RepoError> {
        let e = Email::parse(email).map_err(|_| RepoError::Corrupt("Invalid email in DB".into()))?;
        Ok(Account {
            id: AccountId(id),
            email: e,
            email_verified,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn find_by_email(&self, email: &Email) -> Result<Option<Account>, RepoError> {
        let email_str = email.as_str();
        let row = sqlx::query!(
            r#"
            SELECT id, email, email_verified, created_at, updated_at
            FROM account
            WHERE email = $1
            "#,
            email_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        row.map(|r| {
            Self::map_row(
                r.id,
                &r.email,
                r.email_verified,
                r.created_at,
                r.updated_at,
            )
        })
        .transpose()
    }

    async fn create(&self, email: &Email) -> Result<Account, RepoError> {
        let email_str = email.as_str();
        let row = sqlx::query!(
            r#"
            INSERT INTO account (email)
            VALUES ($1)
            RETURNING id, email, email_verified, created_at, updated_at
            "#,
            email_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;

        Self::map_row(
            row.id,
            &row.email,
            row.email_verified,
            row.created_at,
            row.updated_at,
        )
    }

    async fn mark_verified(&self, account_id: AccountId) -> Result<(), RepoError> {
        let id_uuid = account_id.as_uuid();
        let rows_affected = sqlx::query!(
            r#"
            UPDATE account
            SET email_verified = true, updated_at = now()
            WHERE id = $1
            "#,
            id_uuid
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
