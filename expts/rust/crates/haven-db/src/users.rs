use haven_domain::account::AccountId;
use haven_domain::offer::UserId;
use haven_domain::user::User;

use crate::DbPool;

pub async fn find_by_account_id(
    pool: &DbPool,
    account_id: AccountId,
) -> sqlx::Result<Option<User>> {
    let id_uuid = account_id.as_uuid();
    let row = sqlx::query!(
        r#"
        SELECT id, account_id, created_at, updated_at
        FROM "user"
        WHERE account_id = $1
        "#,
        id_uuid
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: UserId(r.id),
        account_id: AccountId(r.account_id),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

pub async fn create(pool: &DbPool, account_id: AccountId) -> sqlx::Result<User> {
    let id_uuid = account_id.as_uuid();
    let row = sqlx::query!(
        r#"
        INSERT INTO "user" (account_id)
        VALUES ($1)
        RETURNING id, account_id, created_at, updated_at
        "#,
        id_uuid
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: UserId(row.id),
        account_id: AccountId(row.account_id),
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn find_or_create(pool: &DbPool, account_id: AccountId) -> sqlx::Result<User> {
    if let Some(user) = find_by_account_id(pool, account_id).await? {
        Ok(user)
    } else {
        create(pool, account_id).await
    }
}
