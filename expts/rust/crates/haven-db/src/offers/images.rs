//! Database repository implementation for offer image management and ordering.

use async_trait::async_trait;
use haven_domain::image::{ImageDimensions, ImageId, ImagePosition, ImagePublicId, OfferImage};
use haven_domain::offer::{OfferId, UserId};
use haven_domain::ports::{OfferImageRepository, RepoError};
use uuid::Uuid;

use crate::DbPool;

/// PostgreSQL implementation of [`OfferImageRepository`].
pub struct PostgresOfferImageRepository {
    pool: DbPool,
}

impl PostgresOfferImageRepository {
    /// Creates a new `PostgresOfferImageRepository` with a connection pool.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Helper function to load all images for an offer ordered by position ascending.
pub(crate) async fn fetch_images_for_offer(
    pool: &DbPool,
    offer_id: OfferId,
) -> Result<Vec<OfferImage>, RepoError> {
    let o_id = offer_id.as_uuid();
    let rows = sqlx::query!(
        r#"
        SELECT id, offer_id, public_id, position, width, height, created_at
        FROM offer_image
        WHERE offer_id = $1
        ORDER BY position ASC
        "#,
        o_id
    )
    .fetch_all(pool)
    .await
    .map_err(crate::map_sqlx_err)?;

    let mut images = Vec::with_capacity(rows.len());
    for r in rows {
        let pos_u8 = u8::try_from(r.position)
            .map_err(|_| RepoError::Corrupt("invalid image position in database".into()))?;
        let position = ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;
        let public_id =
            ImagePublicId::parse(&r.public_id).map_err(|e| RepoError::Corrupt(e.to_string()))?;
        let width = u32::try_from(r.width)
            .map_err(|_| RepoError::Corrupt("invalid image width in database".into()))?;
        let height = u32::try_from(r.height)
            .map_err(|_| RepoError::Corrupt("invalid image height in database".into()))?;
        let dimensions =
            ImageDimensions::new(width, height).map_err(|e| RepoError::Corrupt(e.to_string()))?;

        images.push(OfferImage {
            id: ImageId::from_uuid(r.id),
            offer_id,
            public_id,
            position,
            dimensions,
            created_at: r.created_at,
        });
    }

    Ok(images)
}

#[async_trait]
impl OfferImageRepository for PostgresOfferImageRepository {
    async fn find_by_offer(&self, offer_id: OfferId) -> Result<Vec<OfferImage>, RepoError> {
        fetch_images_for_offer(&self.pool, offer_id).await
    }

    async fn add_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        public_id: &ImagePublicId,
        dimensions: ImageDimensions,
    ) -> Result<OfferImage, RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();
        let pub_id_str = public_id.as_str();
        let w_val = i32::try_from(dimensions.width())
            .map_err(|_| RepoError::Corrupt("width exceeds integer bounds".into()))?;
        let h_val = i32::try_from(dimensions.height())
            .map_err(|_| RepoError::Corrupt("height exceeds integer bounds".into()))?;

        let mut tx = self.pool.begin().await.map_err(crate::map_sqlx_err)?;

        // Verify offer exists and is owned by caller
        let offer_row = sqlx::query!(
            r#"
            SELECT id, image_count
            FROM offer
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
            o_id,
            u_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "offer",
            id: offer_id.to_string(),
        })?;

        if offer_row.image_count >= 5 {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "maximum 5 images per offer".into(),
            });
        }

        let new_position: i16 = offer_row.image_count.saturating_add(1);
        let new_image_id = Uuid::new_v4();

        let row = sqlx::query!(
            r#"
            INSERT INTO offer_image (id, offer_id, public_id, position, width, height)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, offer_id, public_id, position, width, height, created_at
            "#,
            new_image_id,
            o_id,
            pub_id_str,
            new_position,
            w_val,
            h_val
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

        // Increment offer image_count
        sqlx::query!(
            r#"
            UPDATE offer
            SET image_count = image_count + 1, updated_at = NOW()
            WHERE id = $1
            "#,
            o_id
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

        tx.commit().await.map_err(crate::map_sqlx_err)?;

        let pos_u8 = u8::try_from(row.position)
            .map_err(|_| RepoError::Corrupt("invalid image position".into()))?;
        let position = ImagePosition::new(pos_u8).map_err(|e| RepoError::Corrupt(e.to_string()))?;

        Ok(OfferImage {
            id: ImageId::from_uuid(row.id),
            offer_id,
            public_id: public_id.clone(),
            position,
            dimensions,
            created_at: row.created_at,
        })
    }

    async fn remove_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        image_id: ImageId,
    ) -> Result<ImagePublicId, RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();
        let img_id = image_id.as_uuid();

        let mut tx = self.pool.begin().await.map_err(crate::map_sqlx_err)?;

        // Verify offer exists, caller is owner, and image_count > 1
        let offer_row = sqlx::query!(
            r#"
            SELECT id, image_count
            FROM offer
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
            o_id,
            u_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "offer",
            id: offer_id.to_string(),
        })?;

        if offer_row.image_count <= 1 {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "an offer needs at least one image".into(),
            });
        }

        // Delete image, re-rank remaining positions, and decrement image_count in a single atomic CTE
        let deleted = sqlx::query!(
            r#"
            WITH deleted AS (
                DELETE FROM offer_image
                WHERE id = $1 AND offer_id = $2
                RETURNING public_id
            ),
            reranked AS (
                UPDATE offer_image
                SET position = ranked.new_pos::smallint
                FROM (
                    SELECT id, ROW_NUMBER() OVER (ORDER BY position ASC) AS new_pos
                    FROM offer_image
                    WHERE offer_id = $2 AND id != $1
                ) ranked
                WHERE offer_image.id = ranked.id
            ),
            updated_offer AS (
                UPDATE offer
                SET image_count = image_count - 1, updated_at = NOW()
                WHERE id = $2 AND image_count > 1
            )
            SELECT public_id FROM deleted
            "#,
            img_id,
            o_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "offer_image",
            id: image_id.to_string(),
        })?;

        tx.commit().await.map_err(crate::map_sqlx_err)?;

        ImagePublicId::parse(deleted.public_id).map_err(|e| RepoError::Corrupt(e.to_string()))
    }

    async fn reorder_images(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        order: &[ImageId],
    ) -> Result<Vec<OfferImage>, RepoError> {
        let o_id = offer_id.as_uuid();
        let u_id = user_id.as_uuid();

        let mut tx = self.pool.begin().await.map_err(crate::map_sqlx_err)?;

        // Verify offer exists and caller is owner
        let _offer_row = sqlx::query!(
            r#"
            SELECT id
            FROM offer
            WHERE id = $1 AND user_id = $2
            FOR UPDATE
            "#,
            o_id,
            u_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?
        .ok_or_else(|| RepoError::NotFound {
            entity: "offer",
            id: offer_id.to_string(),
        })?;

        // Fetch existing image IDs for this offer
        let existing = sqlx::query!(
            r#"
            SELECT id
            FROM offer_image
            WHERE offer_id = $1
            "#,
            o_id
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

        if existing.len() != order.len() {
            return Err(RepoError::Conflict {
                entity: "offer_image",
                key: "reorder list length mismatch".into(),
            });
        }

        let existing_set: std::collections::HashSet<Uuid> =
            existing.into_iter().map(|r| r.id).collect();
        for target_id in order {
            if !existing_set.contains(&target_id.as_uuid()) {
                return Err(RepoError::NotFound {
                    entity: "offer_image",
                    id: target_id.to_string(),
                });
            }
        }

        // Apply new positions sequentially (deferred unique constraint is checked on commit)
        for (i, target_id) in order.iter().enumerate() {
            let Ok(pos_val) = i16::try_from(i.saturating_add(1)) else {
                return Err(RepoError::Corrupt("position overflow".into()));
            };
            let img_id = target_id.as_uuid();

            sqlx::query!(
                r#"
                UPDATE offer_image
                SET position = $2
                WHERE id = $1
                "#,
                img_id,
                pos_val
            )
            .execute(&mut *tx)
            .await
            .map_err(crate::map_sqlx_err)?;
        }

        sqlx::query!(
            r#"
            UPDATE offer
            SET updated_at = NOW()
            WHERE id = $1
            "#,
            o_id
        )
        .execute(&mut *tx)
        .await
        .map_err(crate::map_sqlx_err)?;

        tx.commit().await.map_err(crate::map_sqlx_err)?;

        fetch_images_for_offer(&self.pool, offer_id).await
    }
}
