use crate::account::{Account, AccountId, Email};
use crate::magic_link::MagicLink;
use crate::offer::{CreateOffer, Offer, OfferId, UpdateOffer, UserId};
use crate::session::{HashedToken, Session, SessionId};
use crate::user::User;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::net::IpAddr;
use thiserror::Error;
use uuid::Uuid;

/// A strongly typed newtype for idempotency keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(pub Uuid);

/// Infrastructure failures and database mapping errors.
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Resource not found or unauthorized")]
    NotFound,
    #[error("Conflict with existing resource")]
    Conflict,
    #[error("Data corruption: {0}")]
    Corrupt(String),
    #[error("Database unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait OfferRepository: Send + Sync {
    async fn find_owned(
        &self,
        offer_id: OfferId,
        user_id: UserId,
    ) -> Result<Option<Offer>, RepoError>;
    
    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Offer>, RepoError>;
    
    async fn create(
        &self,
        user_id: UserId,
        key: IdempotencyKey,
        offer: &CreateOffer,
    ) -> Result<Offer, RepoError>;
    
    async fn update(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        offer: &UpdateOffer,
    ) -> Result<Offer, RepoError>;
    
    async fn delete(&self, offer_id: OfferId, user_id: UserId) -> Result<(), RepoError>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_email(&self, email: &Email) -> Result<Option<Account>, RepoError>;
    async fn create(&self, email: &Email) -> Result<Account, RepoError>;
    async fn mark_verified(&self, account_id: AccountId) -> Result<(), RepoError>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_account(&self, account_id: AccountId) -> Result<Option<User>, RepoError>;
    async fn create(&self, account_id: AccountId) -> Result<User, RepoError>;
    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError>;
}

#[async_trait]
pub trait MagicLinkRepository: Send + Sync {
    async fn consume(
        &self,
        token_hash: &HashedToken,
    ) -> Result<Option<MagicLink>, RepoError>;
    
    async fn create(
        &self,
        email: &Email,
        token_hash: &HashedToken,
        expires_at: DateTime<Utc>,
    ) -> Result<MagicLink, RepoError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_token_hash(
        &self,
        token_hash: &HashedToken,
    ) -> Result<Option<Session>, RepoError>;
    
    async fn create(
        &self,
        account_id: AccountId,
        token_hash: &HashedToken,
        expires_at: DateTime<Utc>,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<Session, RepoError>;
    
    async fn delete(&self, session_id: SessionId) -> Result<(), RepoError>;
}

/// A single composition root.
pub trait Registry: Send + Sync {
    fn offers(&self) -> &(dyn OfferRepository + 'static);
    fn accounts(&self) -> &(dyn AccountRepository + 'static);
    fn users(&self) -> &(dyn UserRepository + 'static);
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static);
    fn sessions(&self) -> &(dyn SessionRepository + 'static);
}
