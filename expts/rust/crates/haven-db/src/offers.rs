//! PostgreSQL implementation of `OfferRepository` and submodule assembly.

pub mod feed;
pub mod queries;
pub mod rows;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::manual_let_else,
    reason = "test assertions and optional local db connectivity"
)]
mod tests;

use crate::DbPool;
use async_trait::async_trait;
use haven_domain::offer::{
    CreateOffer, Offer, OfferCursor, OfferId, OfferPage, OfferSlug, UpdateOffer, UserId,
};
use haven_domain::ports::{IdempotencyKey, OfferRepository, RepoError};

/// PostgreSQL-backed repository for offers.
pub struct PostgresOfferRepository {
    /// PostgreSQL connection pool.
    pub pool: DbPool,
}

#[async_trait]
impl OfferRepository for PostgresOfferRepository {
    async fn find_owned(
        &self,
        offer_id: OfferId,
        user_id: UserId,
    ) -> Result<Option<Offer>, RepoError> {
        queries::find_owned(&self.pool, offer_id, user_id).await
    }

    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Offer>, RepoError> {
        queries::find_by_user(&self.pool, user_id).await
    }

    async fn find_by_slug(&self, slug: &OfferSlug) -> Result<Option<Offer>, RepoError> {
        queries::find_by_slug(&self.pool, slug).await
    }

    async fn list_public(
        &self,
        cursor: Option<&OfferCursor>,
        limit: usize,
    ) -> Result<OfferPage, RepoError> {
        feed::list_public(&self.pool, cursor, limit).await
    }

    async fn create(
        &self,
        user_id: UserId,
        key: IdempotencyKey,
        create_offer: &CreateOffer,
    ) -> Result<Offer, RepoError> {
        queries::create(&self.pool, user_id, key, create_offer).await
    }

    async fn update(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        update_offer: &UpdateOffer,
    ) -> Result<Offer, RepoError> {
        queries::update(&self.pool, offer_id, user_id, update_offer).await
    }

    async fn delete(&self, offer_id: OfferId, user_id: UserId) -> Result<(), RepoError> {
        queries::delete(&self.pool, offer_id, user_id).await
    }
}
