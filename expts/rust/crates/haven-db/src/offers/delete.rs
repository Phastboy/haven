use haven_domain::offer::OfferId;
use crate::DbPool;

pub async fn delete(pool: &DbPool, offer_id: OfferId) -> sqlx::Result<()> {
    let o_id = offer_id.as_uuid();
    sqlx::query!(
        r#"
        DELETE FROM offer
        WHERE id = $1
        "#,
        o_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
