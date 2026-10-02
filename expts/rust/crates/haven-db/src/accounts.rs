use haven_domain::account::{Account, AccountId, Email};

use crate::DbPool;

pub async fn find_by_email(pool: &DbPool, email: &Email) -> sqlx::Result<Option<Account>> {
    let email_str = email.as_str();
    let row = sqlx::query!(
        r#"
        SELECT id, email, email_verified, created_at, updated_at
        FROM account
        WHERE email = $1
        "#,
        email_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Account {
        id: AccountId(r.id),
        email: Email::parse(&r.email).expect("db email must be valid"),
        email_verified: r.email_verified,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

pub async fn create(pool: &DbPool, email: &Email) -> sqlx::Result<Account> {
    let email_str = email.as_str();
    let row = sqlx::query!(
        r#"
        INSERT INTO account (email)
        VALUES ($1)
        RETURNING id, email, email_verified, created_at, updated_at
        "#,
        email_str
    )
    .fetch_one(pool)
    .await?;

    Ok(Account {
        id: AccountId(row.id),
        email: Email::parse(&row.email).expect("db email must be valid"),
        email_verified: row.email_verified,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn mark_verified(pool: &DbPool, account_id: AccountId) -> sqlx::Result<()> {
    let id_uuid = account_id.as_uuid();
    sqlx::query!(
        r#"
        UPDATE account
        SET email_verified = true, updated_at = now()
        WHERE id = $1
        "#,
        id_uuid
    )
    .execute(pool)
    .await?;
    Ok(())
}
