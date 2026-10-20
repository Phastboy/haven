//! Row models and mapping for conversations and messages.

use chrono::{DateTime, Utc};
use haven_domain::conversation::{
    Conversation, ConversationId, ConversationInboxItem, ConversationPair, Message, MessageBody,
    MessageId,
};
use haven_domain::offer::OfferId;
use haven_domain::ports::RepoError;
use haven_domain::user::{DisplayName, ProfileHandle, UserId};
use uuid::Uuid;

pub struct ConversationRow {
    pub id: Uuid,
    pub user_low: Uuid,
    pub user_high: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
}

impl ConversationRow {
    pub fn into_domain(self) -> Result<Conversation, RepoError> {
        let low = UserId::from_uuid(self.user_low);
        let high = UserId::from_uuid(self.user_high);
        let pair = ConversationPair::new(low, high)
            .map_err(|e| RepoError::Corrupt(format!("Corrupt conversation pair in DB: {e}")))?;

        Ok(Conversation {
            id: ConversationId::from_uuid(self.id),
            pair,
            created_at: self.created_at,
            last_message_at: self.last_message_at,
        })
    }
}

pub struct MessageRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub offer_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl MessageRow {
    pub fn into_domain(self) -> Result<Message, RepoError> {
        let body = MessageBody::parse(self.body)
            .map_err(|e| RepoError::Corrupt(format!("Corrupt message body in DB: {e}")))?;

        Ok(Message {
            id: MessageId::from_uuid(self.id),
            conversation_id: ConversationId::from_uuid(self.conversation_id),
            sender_id: UserId::from_uuid(self.sender_id),
            body,
            offer_id: self.offer_id.map(OfferId::from_uuid),
            created_at: self.created_at,
        })
    }
}

pub struct InboxRow {
    pub conversation_id: Uuid,
    pub other_user_id: Uuid,
    pub other_display_name: String,
    pub other_handle: String,
    pub other_avatar_public_id: Option<String>,
    pub last_message_body: Option<String>,
    pub last_message_at: DateTime<Utc>,
    pub is_unread: bool,
}

impl InboxRow {
    pub fn into_domain(self) -> Result<ConversationInboxItem, RepoError> {
        let other_display_name = DisplayName::parse(&self.other_display_name)
            .map_err(|e| RepoError::Corrupt(format!("Corrupt display name in DB: {e}")))?;
        let other_handle = ProfileHandle::parse(&self.other_handle)
            .map_err(|e| RepoError::Corrupt(format!("Corrupt handle in DB: {e}")))?;

        Ok(ConversationInboxItem {
            id: ConversationId::from_uuid(self.conversation_id),
            other_user_id: UserId::from_uuid(self.other_user_id),
            other_display_name,
            other_handle,
            other_avatar_public_id: self.other_avatar_public_id,
            last_message_body: self.last_message_body.unwrap_or_default(),
            last_message_at: self.last_message_at,
            is_unread: self.is_unread,
        })
    }
}
