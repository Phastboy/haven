use crate::account::{Account, AccountId, Email};
use crate::conversation::{
    Conversation, ConversationId, ConversationInboxPage, ConversationPair, Message, MessagePage,
    NewMessage, OfferInquirersPage,
};
use crate::image::{ImageDimensions, ImageId, ImagePublicId, OfferImage};
use crate::magic_link::MagicLink;
use crate::offer::{
    CreateOffer, Offer, OfferCursor, OfferId, OfferPage, OfferSlug, PublicOfferPage, UpdateOffer,
    UserId,
};
use crate::saved::{CollectionCursor, FollowPage, SavedOfferPage};
use crate::session::{HashedToken, Session, SessionId};
use crate::user::{ProfileHandle, User, UserProfileUpdate};
pub use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{collections::HashSet, net::IpAddr};
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

/// Distinguishes between a newly created resource and a replayed idempotent request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreationStatus<T> {
    /// The resource was newly created and persisted.
    Created(T),
    /// The resource already existed for this idempotency key; the existing state was returned.
    Replayed(T),
}

impl<T> CreationStatus<T> {
    /// Extracts the inner resource regardless of whether it was created or replayed.
    #[must_use]
    pub fn into_inner(self) -> T {
        match self {
            Self::Created(v) | Self::Replayed(v) => v,
        }
    }

    /// Returns `true` if the resource was newly created.
    #[must_use]
    pub fn is_new(&self) -> bool {
        matches!(self, Self::Created(_))
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
        query: &crate::offer::PublicFeedQuery,
    ) -> Result<PublicOfferPage, RepoError>;

    async fn list_by_user(
        &self,
        user_id: UserId,
        cursor: Option<&OfferCursor>,
        limit: usize,
    ) -> Result<OfferPage, RepoError>;

    async fn create(
        &self,
        user_id: UserId,
        key: IdempotencyKey,
        offer: &CreateOffer,
    ) -> Result<CreationStatus<Offer>, RepoError>;

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
    async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>, RepoError>;
    async fn find_by_handle(&self, handle: &ProfileHandle) -> Result<Option<User>, RepoError>;
    async fn create(
        &self,
        account_id: AccountId,
        profile: &UserProfileUpdate,
    ) -> Result<User, RepoError>;
    async fn update_profile(
        &self,
        user_id: UserId,
        profile: &UserProfileUpdate,
    ) -> Result<User, RepoError>;
    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError>;
    async fn update_avatar(
        &self,
        user_id: UserId,
        avatar_public_id: Option<&ImagePublicId>,
    ) -> Result<User, RepoError>;
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

/// Result metadata returned after a successful image upload to an external host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredImage {
    /// Opaque public ID assigned by the host (e.g. `offers/{offer_id}/{uuid}`).
    pub public_id: String,
    /// Width of the stored image in pixels.
    pub width: u32,
    /// Height of the stored image in pixels.
    pub height: u32,
    /// Format identifier of the stored asset (e.g. "webp", "jpg").
    pub format: String,
    /// Size of the stored asset in bytes.
    pub bytes: usize,
}

/// Errors occurring during image storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("image upload failed: {0}")]
    UploadFailed(String),

    #[error("image deletion failed: {0}")]
    DeleteFailed(String),

    #[error("image storage service unavailable: {0}")]
    Unavailable(String),
}

/// Secondary port for uploading and deleting image assets from external storage.
#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Image upload takes &self, target folder, public ID suffix, and raw image byte slice"
)]
pub trait ImageStorage: Send + Sync {
    /// Uploads raw image bytes to a specified storage folder with a given public ID suffix.
    async fn upload(
        &self,
        folder: &str,
        public_id_suffix: &str,
        bytes: &[u8],
    ) -> Result<StoredImage, StorageError>;

    /// Deletes an image asset from storage by its public ID.
    async fn delete(&self, public_id: &str) -> Result<(), StorageError>;
}

/// Repository port for managing offer images and their ordering.
#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Repository port trait methods take &self alongside domain arguments"
)]
pub trait OfferImageRepository: Send + Sync {
    /// Returns all images for a given offer, ordered by position (1..=5).
    async fn find_by_offer(&self, offer_id: OfferId) -> Result<Vec<OfferImage>, RepoError>;

    /// Adds an image to an offer, verifying ownership, enforcing maximum of 5 images,
    /// assigning the next sequential position, and incrementing offer `image_count`.
    async fn add_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        public_id: &ImagePublicId,
        dimensions: ImageDimensions,
    ) -> Result<OfferImage, RepoError>;

    /// Removes an image from an offer by its image ID, verifying ownership,
    /// compacting remaining image positions (1..=N), and decrementing offer `image_count`.
    /// Returns the deleted image's `ImagePublicId` so external storage can be purged.
    async fn remove_image(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        image_id: ImageId,
    ) -> Result<ImagePublicId, RepoError>;

    /// Reorders images for an offer, ensuring the order contains all and only existing images,
    /// updating positions 1..=N.
    async fn reorder_images(
        &self,
        offer_id: OfferId,
        user_id: UserId,
        order: &[ImageId],
    ) -> Result<Vec<OfferImage>, RepoError>;
}

/// Repository port for conversations and messages.
#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Repository port methods take &self alongside query and pagination parameters"
)]
pub trait ConversationRepository: Send + Sync {
    /// Gets an existing conversation for the canonical pair or creates a new one.
    async fn get_or_create(&self, pair: ConversationPair) -> Result<Conversation, RepoError>;

    /// Finds a conversation by its unique ID.
    async fn find_by_id(&self, id: ConversationId) -> Result<Option<Conversation>, RepoError>;

    /// Appends a new message to a conversation and updates conversation's `last_message_at`
    /// as well as the sender's participant `last_read_at`.
    async fn append(
        &self,
        conversation_id: ConversationId,
        new_message: NewMessage,
    ) -> Result<Message, RepoError>;

    /// Lists conversations for a user, sorted by `last_message_at DESC` with keyset pagination.
    async fn list_for_user(
        &self,
        user_id: UserId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<ConversationInboxPage, RepoError>;

    /// Fetches messages in a conversation with keyset pagination.
    async fn messages(
        &self,
        conversation_id: ConversationId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<MessagePage, RepoError>;

    /// Updates the participant's `last_read_at` to the current timestamp.
    async fn mark_read(
        &self,
        conversation_id: ConversationId,
        user_id: UserId,
    ) -> Result<(), RepoError>;

    /// Checks if any unread messages exist across all conversations for a user.
    async fn unread_exists(&self, user_id: UserId) -> Result<bool, RepoError>;

    /// Lists inquirers who messaged about a specific offer (v0.4.1).
    async fn list_offer_inquirers(
        &self,
        offer_id: OfferId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<OfferInquirersPage, RepoError>;
}

/// Persistence boundary for saving offers and following people.
#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Repository signatures accept entity identifiers and a pagination cursor"
)]
pub trait SavedOfferRepository: Send + Sync {
    /// Checks whether an offer is currently saved.
    async fn is_saved(&self, user_id: UserId, offer_id: OfferId) -> Result<bool, RepoError>;
    /// Returns saved offer IDs from the supplied page in one repository operation.
    async fn saved_among(
        &self,
        user_id: UserId,
        offer_ids: &[OfferId],
    ) -> Result<HashSet<OfferId>, RepoError>;
    /// Saves an offer; repeated saves are successful and leave the original timestamp.
    async fn save(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError>;
    /// Removes a saved offer; repeated removals are successful.
    async fn unsave(&self, user_id: UserId, offer_id: OfferId) -> Result<(), RepoError>;
    /// Lists saved offers by reverse creation time. Missing or private offers remain in the list
    /// with `offer` set to `None`.
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<SavedOfferPage, RepoError>;
}

/// Persistence boundary for following people.
#[async_trait]
#[allow(
    clippy::too_many_arguments,
    reason = "Repository signatures accept entity identifiers and a pagination cursor"
)]
pub trait FollowRepository: Send + Sync {
    /// Checks whether a person is currently followed.
    async fn is_following(&self, user_id: UserId, followed_id: UserId) -> Result<bool, RepoError>;
    /// Follows a person idempotently; rejects self-follow.
    async fn follow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError>;
    /// Unfollows a person idempotently.
    async fn unfollow(&self, user_id: UserId, followed_id: UserId) -> Result<(), RepoError>;
    /// Lists followed people by reverse creation time.
    async fn list(
        &self,
        user_id: UserId,
        cursor: Option<CollectionCursor>,
        limit: usize,
    ) -> Result<FollowPage, RepoError>;
}

/// A single composition root.
pub trait Registry: Send + Sync {
    fn offers(&self) -> &(dyn OfferRepository + 'static);
    fn accounts(&self) -> &(dyn AccountRepository + 'static);
    fn users(&self) -> &(dyn UserRepository + 'static);
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static);
    fn sessions(&self) -> &(dyn SessionRepository + 'static);
    fn offer_images(&self) -> &(dyn OfferImageRepository + 'static);
    fn conversations(&self) -> &(dyn ConversationRepository + 'static);
    fn saved_offers(&self) -> &(dyn SavedOfferRepository + 'static);
    fn follows(&self) -> &(dyn FollowRepository + 'static);
}
