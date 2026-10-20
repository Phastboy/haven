//! Conversation and messaging domain models.

pub mod cursor;
#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::error::DomainError;
use crate::offer::OfferId;
use crate::user::UserId;

pub use cursor::{
    ConversationCursor, ConversationInboxItem, ConversationInboxPage, MessageCursor, MessagePage,
};

/// Error returned when trying to create a conversation with oneself.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
#[error("cannot create conversation with oneself")]
pub struct SelfConversation;

/// Error returned when a message body fails domain validation.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum MessageValidationError {
    #[error("message body cannot be blank")]
    BlankBody,
    #[error("message body exceeds maximum allowed length of 2000 characters")]
    BodyTooLong,
}

impl From<SelfConversation> for DomainError {
    fn from(err: SelfConversation) -> Self {
        // We will add conversation errors to DomainError
        DomainError::Conversation(ConversationDomainError::SelfConversation(err))
    }
}

/// Umbrella domain error for conversations.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ConversationDomainError {
    #[error(transparent)]
    SelfConversation(#[from] SelfConversation),
    #[error(transparent)]
    MessageValidation(#[from] MessageValidationError),
}

/// Strongly typed Conversation identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(Uuid);

impl ConversationId {
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for ConversationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Strongly typed Message identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Two distinct people, stored in a canonical order so that
/// (a, b) and (b, a) are the same pair. The only constructor
/// enforces `low < high`, so an unordered or self pair cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationPair {
    low: UserId,
    high: UserId,
}

impl ConversationPair {
    /// Returns an error if both ids are equal.
    pub fn new(a: UserId, b: UserId) -> Result<Self, SelfConversation> {
        if a == b {
            return Err(SelfConversation);
        }
        if a.as_uuid() < b.as_uuid() {
            Ok(Self { low: a, high: b })
        } else {
            Ok(Self { low: b, high: a })
        }
    }

    #[must_use]
    pub fn low(&self) -> UserId {
        self.low
    }

    #[must_use]
    pub fn high(&self) -> UserId {
        self.high
    }

    /// Returns the other user in the pair, or `None` if `me` is not in this pair.
    #[must_use]
    pub fn other(&self, me: UserId) -> Option<UserId> {
        if me == self.low {
            Some(self.high)
        } else if me == self.high {
            Some(self.low)
        } else {
            None
        }
    }

    /// Checks if a user is a participant in this pair.
    #[must_use]
    pub fn contains(&self, user_id: UserId) -> bool {
        self.low == user_id || self.high == user_id
    }
}

/// Validated message body (non-blank, <= 2,000 characters).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageBody(String);

impl MessageBody {
    pub const MAX_CHARS: usize = 2000;

    /// Validates and parses a message body.
    pub fn parse(s: impl AsRef<str>) -> Result<Self, MessageValidationError> {
        let trimmed = s.as_ref().trim();
        if trimmed.is_empty() {
            return Err(MessageValidationError::BlankBody);
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(MessageValidationError::BodyTooLong);
        }
        Ok(Self(trimmed.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for MessageBody {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MessageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Core Conversation entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub pair: ConversationPair,
    pub created_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
}

/// Core Message entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    pub sender_id: UserId,
    pub body: MessageBody,
    pub offer_id: Option<OfferId>,
    pub created_at: DateTime<Utc>,
}

/// Grouped input for appending a new message.
#[derive(Debug, Clone)]
pub struct NewMessage {
    pub sender_id: UserId,
    pub body: MessageBody,
    pub offer_id: Option<OfferId>,
}

/// Askers who inquired about an offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferInquirer {
    pub user_id: UserId,
    pub conversation_id: ConversationId,
    pub last_inquiry_at: DateTime<Utc>,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferInquirersPage {
    pub items: Vec<OfferInquirer>,
    pub next_cursor: Option<String>,
}
