//! PostgreSQL persistence for saved offers and follows.

use crate::DbPool;
use async_trait::async_trait;
use haven_domain::{
    offer::OfferId,
    ports::{FollowRepository, RepoError, SavedOfferRepository},
    saved::{CollectionCursor, FollowPage, FollowedPerson, SavedOffer, SavedOfferPage},
    user::UserId,
};

#[cfg(test)]
mod tests;

/// PostgreSQL saved-offer repository.
pub struct PostgresSavedOfferRepository {
    pool: DbPool,
}
impl PostgresSavedOfferRepository {
    /// Creates a repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SavedOfferRepository for PostgresSavedOfferRepository {
    async fn is_saved(&self, user_id: UserId, offer_id: OfferId) -> Result<bool, RepoError> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM saved_offer WHERE user_id = $1 AND offer_id = $2)",
        )
        .bind(user_id.as_uuid())
        .bind(offer_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;
        Ok(sqlx::Row::get::<bool, _>(&row, 0))
    }
    async fn save(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError> {
        sqlx::query("INSERT INTO saved_offer (user_id, offer_id) SELECT $1, id FROM offer WHERE id = $2 AND image_count >= 1 ON CONFLICT (user_id, offer_id) DO NOTHING")
            .bind(user_id.as_uuid()).bind(offer_id.as_uuid()).execute(&self.pool).await.map_err(crate::map_sqlx_err)?;
        Ok(())
    }
    async fn unsave(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM saved_offer WHERE user_id = $1 AND offer_id = $2")
            .bind(user_id.as_uuid())
            .bind(offer_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(crate::map_sqlx_err)?;
        Ok(())
    }
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<SavedOfferPage, RepoError> {
        let limit = i64::try_from(limit.clamp(1, 50))
            .unwrap_or(50)
            .saturating_add(1);
        let query = if cursor.is_some() {
            "SELECT s.id AS saved_id, s.created_at AS saved_at, o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.image_count, o.created_at, o.updated_at FROM saved_offer s LEFT JOIN offer o ON o.id = s.offer_id WHERE s.user_id = $1 AND (s.created_at, s.id) < ($2, $3) ORDER BY s.created_at DESC, s.id DESC LIMIT $4"
        } else {
            "SELECT s.id AS saved_id, s.created_at AS saved_at, o.id, o.user_id, o.slug, o.title, o.description, o.price, o.currency, o.image_count, o.created_at, o.updated_at FROM saved_offer s LEFT JOIN offer o ON o.id = s.offer_id WHERE s.user_id = $1 ORDER BY s.created_at DESC, s.id DESC LIMIT $2"
        };
        let mut q = sqlx::query(query).bind(user_id.as_uuid());
        if let Some(c) = cursor {
            q = q.bind(c.created_at).bind(c.id).bind(limit);
        } else {
            q = q.bind(limit);
        }
        let rows = q.fetch_all(&self.pool).await.map_err(crate::map_sqlx_err)?;
        let has_more = rows.len() >= usize::try_from(limit).unwrap_or(51);
        let mut items = Vec::new();
        for row in rows
            .into_iter()
            .take(usize::try_from(limit.saturating_sub(1)).unwrap_or(50))
        {
            let id: uuid::Uuid = sqlx::Row::get(&row, "saved_id");
            let created_at = sqlx::Row::get(&row, "saved_at");
            let offer_id: Option<uuid::Uuid> = sqlx::Row::get(&row, "id");
            let offer = if let Some(offer_id) = offer_id {
                let image_count: Option<i16> = sqlx::Row::get(&row, "image_count");
                if image_count.unwrap_or_default() > 0 {
                    Some(crate::offers::rows::map_row(
                        offer_id,
                        sqlx::Row::get(&row, "user_id"),
                        sqlx::Row::get(&row, "slug"),
                        sqlx::Row::get(&row, "title"),
                        sqlx::Row::get(&row, "description"),
                        sqlx::Row::get(&row, "price"),
                        sqlx::Row::get(&row, "currency"),
                        image_count.unwrap_or_default(),
                        Vec::new(),
                        sqlx::Row::get(&row, "created_at"),
                        sqlx::Row::get(&row, "updated_at"),
                    )?)
                } else {
                    None
                }
            } else {
                None
            };
            let offer_id = offer
                .as_ref()
                .map(|o| o.id)
                .or_else(|| {
                    sqlx::Row::get::<Option<uuid::Uuid>, _>(&row, "id").map(OfferId::from_uuid)
                })
                .ok_or_else(|| RepoError::Corrupt("saved offer lost its offer id".into()))?;
            items.push(SavedOffer {
                offer_id,
                offer,
                created_at,
                id,
            });
        }
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| CollectionCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };
        Ok(SavedOfferPage { items, next_cursor })
    }
}

/// PostgreSQL follow repository.
pub struct PostgresFollowRepository {
    pool: DbPool,
}
impl PostgresFollowRepository {
    /// Creates a repository.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FollowRepository for PostgresFollowRepository {
    async fn is_following(&self, user_id: UserId, followed_id: UserId) -> Result<bool, RepoError> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM follow WHERE user_id = $1 AND followed_id = $2)",
        )
        .bind(user_id.as_uuid())
        .bind(followed_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(crate::map_sqlx_err)?;
        Ok(sqlx::Row::get::<bool, _>(&row, 0))
    }
    async fn follow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError> {
        if user_id == followed_id {
            return Err(RepoError::Conflict {
                entity: "follow",
                key: "cannot follow yourself".into(),
            });
        }
        sqlx::query("INSERT INTO follow (user_id, followed_id) VALUES ($1, $2) ON CONFLICT (user_id, followed_id) DO NOTHING")
            .bind(user_id.as_uuid()).bind(followed_id.as_uuid()).execute(&self.pool).await.map_err(crate::map_sqlx_err)?;
        Ok(())
    }
    async fn unfollow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM follow WHERE user_id = $1 AND followed_id = $2")
            .bind(user_id.as_uuid())
            .bind(followed_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(crate::map_sqlx_err)?;
        Ok(())
    }
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<FollowPage, RepoError> {
        let limit = i64::try_from(limit.clamp(1, 50))
            .unwrap_or(50)
            .saturating_add(1);
        let query = if cursor.is_some() {
            "SELECT f.id AS follow_id, f.created_at AS followed_at, u.id, u.account_id, u.handle, u.display_name, u.bio, u.avatar_public_id, u.created_at, u.updated_at FROM follow f JOIN \"user\" u ON u.id = f.followed_id WHERE f.user_id = $1 AND (f.created_at, f.id) < ($2, $3) ORDER BY f.created_at DESC, f.id DESC LIMIT $4"
        } else {
            "SELECT f.id AS follow_id, f.created_at AS followed_at, u.id, u.account_id, u.handle, u.display_name, u.bio, u.avatar_public_id, u.created_at, u.updated_at FROM follow f JOIN \"user\" u ON u.id = f.followed_id WHERE f.user_id = $1 ORDER BY f.created_at DESC, f.id DESC LIMIT $2"
        };
        let mut q = sqlx::query(query).bind(user_id.as_uuid());
        if let Some(c) = cursor {
            q = q.bind(c.created_at).bind(c.id).bind(limit);
        } else {
            q = q.bind(limit);
        }
        let rows = q.fetch_all(&self.pool).await.map_err(crate::map_sqlx_err)?;
        let has_more = rows.len() >= usize::try_from(limit).unwrap_or(51);
        let mut items = Vec::new();
        for row in rows
            .into_iter()
            .take(usize::try_from(limit.saturating_sub(1)).unwrap_or(50))
        {
            let person = crate::users::PostgresUserRepository::map_row(crate::users::UserDbRow {
                id: sqlx::Row::get(&row, "id"),
                account_id: sqlx::Row::get(&row, "account_id"),
                handle: sqlx::Row::get(&row, "handle"),
                display_name: sqlx::Row::get(&row, "display_name"),
                bio: sqlx::Row::get(&row, "bio"),
                avatar_public_id: sqlx::Row::get(&row, "avatar_public_id"),
                created_at: sqlx::Row::get(&row, "created_at"),
                updated_at: sqlx::Row::get(&row, "updated_at"),
            })?;
            items.push(FollowedPerson {
                person,
                created_at: sqlx::Row::get(&row, "followed_at"),
                id: sqlx::Row::get(&row, "follow_id"),
            });
        }
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| CollectionCursor::new(item.created_at, item.id).encode())
        } else {
            None
        };
        Ok(FollowPage { items, next_cursor })
    }
}
