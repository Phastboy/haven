use haven_domain::session::SessionId;
use crate::DbPool;

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
