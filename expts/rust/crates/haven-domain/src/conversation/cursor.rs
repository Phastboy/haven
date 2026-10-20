//! Cursors and paginated structures for conversations and messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ConversationId, MessageId};
use crate::error::CursorError;
use crate::user::{DisplayName, ProfileHandle, UserId};

/// Inbox list item representing a conversation preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInboxItem {
    pub id: ConversationId,
    pub other_user_id: UserId,
    pub other_display_name: DisplayName,
    pub other_handle: ProfileHandle,
    pub other_avatar_public_id: Option<String>,
    pub last_message_body: String,
    pub last_message_at: DateTime<Utc>,
    pub is_unread: bool,
}

/// Paginated page of conversation previews for the inbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInboxPage {
    pub items: Vec<ConversationInboxItem>,
    pub next_cursor: Option<String>,
}

/// Cursor for paginating conversations (descending by `last_message_at`, tie-breaker `conversation_id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationCursor {
    pub last_message_at: DateTime<Utc>,
    pub id: ConversationId,
}

impl ConversationCursor {
    #[must_use]
    pub fn new(last_message_at: DateTime<Utc>, id: ConversationId) -> Self {
        Self {
            last_message_at,
            id,
        }
    }

    #[must_use]
    pub fn encode(&self) -> String {
        let raw = format!(
            "{}_{}",
            self.last_message_at.timestamp_micros(),
            self.id.as_uuid()
        );
        hex::encode(raw.as_bytes())
    }

    pub fn decode(encoded: &str) -> Result<Self, CursorError> {
        let bytes = hex::decode(encoded).map_err(|_| CursorError::InvalidCursor)?;
        let raw = String::from_utf8(bytes).map_err(|_| CursorError::InvalidCursor)?;
        let mut parts = raw.split('_');
        let micros_str = parts.next().ok_or(CursorError::InvalidCursor)?;
        let uuid_str = parts.next().ok_or(CursorError::InvalidCursor)?;
        if parts.next().is_some() {
            return Err(CursorError::InvalidCursor);
        }
        let micros: i64 = micros_str.parse().map_err(|_| CursorError::InvalidCursor)?;
        let uuid: uuid::Uuid = uuid_str.parse().map_err(|_| CursorError::InvalidCursor)?;
        let created_at =
            DateTime::from_timestamp_micros(micros).ok_or(CursorError::InvalidCursor)?;
        Ok(Self {
            last_message_at: created_at,
            id: ConversationId::from_uuid(uuid),
        })
    }
}

/// Keyset cursor for thread messages (`created_at`, `message_id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageCursor {
    pub created_at: DateTime<Utc>,
    pub id: MessageId,
}

impl MessageCursor {
    #[must_use]
    pub fn new(created_at: DateTime<Utc>, id: MessageId) -> Self {
        Self { created_at, id }
    }

    #[must_use]
    pub fn encode(&self) -> String {
        let raw = format!(
            "{}_{}",
            self.created_at.timestamp_micros(),
            self.id.as_uuid()
        );
        hex::encode(raw.as_bytes())
    }

    pub fn decode(encoded: &str) -> Result<Self, CursorError> {
        let bytes = hex::decode(encoded).map_err(|_| CursorError::InvalidCursor)?;
        let raw = String::from_utf8(bytes).map_err(|_| CursorError::InvalidCursor)?;
        let mut parts = raw.split('_');
        let micros_str = parts.next().ok_or(CursorError::InvalidCursor)?;
        let uuid_str = parts.next().ok_or(CursorError::InvalidCursor)?;
        if parts.next().is_some() {
            return Err(CursorError::InvalidCursor);
        }
        let micros: i64 = micros_str.parse().map_err(|_| CursorError::InvalidCursor)?;
        let uuid: uuid::Uuid = uuid_str.parse().map_err(|_| CursorError::InvalidCursor)?;
        let created_at =
            DateTime::from_timestamp_micros(micros).ok_or(CursorError::InvalidCursor)?;
        Ok(Self {
            created_at,
            id: MessageId::from_uuid(uuid),
        })
    }
}

/// Paginated page of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePage {
    pub items: Vec<super::Message>,
    pub next_cursor: Option<String>,
}
