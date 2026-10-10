use crate::account::{Account, AccountId, Email};
use crate::magic_link::MagicLink;
use crate::offer::{
    CreateOffer, Offer, OfferCursor, OfferId, OfferPage, OfferSlug, UpdateOffer, UserId,
};
use crate::session::{HashedToken, Session, SessionId};
use crate::user::User;
pub use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::net::IpAddr;
use thiserror::Error;
use uuid::Uuid;

/// A strongly typed newtype for idempotency keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(Uuid);

impl IdempotencyKey {
    /// Constructs a new idempotency key from a UUID.
    #[must_use]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Constructs an idempotency key from an existing UUID.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying raw UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for IdempotencyKey {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Infrastructure failures and database mapping errors.
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },
    #[error("{entity} conflict on key: {key}")]
    Conflict { entity: &'static str, key: String },
    #[error("Data corruption: {0}")]
    Corrupt(String),
    #[error("Database unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Repository port trait methods take &self alongside up to 3 domain arguments"
)]
pub trait OfferRepository: Send + Sync {
    async fn find_owned(
        &self,
        offer_id: OfferId,
        user_id: UserId,
    ) -> Result<Option<Offer>, RepoError>;

    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Offer>, RepoError>;

    async fn find_by_slug(&self, slug: &OfferSlug) -> Result<Option<Offer>, RepoError>;

    async fn list_public(
        &self,
        cursor: Option<&OfferCursor>,
        limit: usize,
    ) -> Result<OfferPage, RepoError>;

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
#[allow(
    clippy::too_many_arguments,
    reason = "Repository port trait methods take &self alongside up to 3 domain arguments"
)]
pub trait MagicLinkRepository: Send + Sync {
    async fn consume(&self, token_hash: &HashedToken) -> Result<Option<MagicLink>, RepoError>;

    async fn create(
        &self,
        email: &Email,
        token_hash: &HashedToken,
        expires_at: DateTime<Utc>,
    ) -> Result<MagicLink, RepoError>;
}

#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Session creation takes &self, account, token hash, expiration, IP, and user-agent metadata"
)]
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
