//! PostgreSQL implementation of `ConversationRepository`.

pub mod queries;
pub mod rows;

#[cfg(test)]
mod tests;

use crate::DbPool;
use async_trait::async_trait;
use haven_domain::conversation::{
    Conversation, ConversationId, ConversationInboxPage, ConversationPair, Message, MessagePage,
    NewMessage, OfferInquirersPage,
};
use haven_domain::offer::OfferId;
use haven_domain::ports::{ConversationRepository, RepoError};
use haven_domain::user::UserId;

/// PostgreSQL-backed repository for conversations and messages.
pub struct PostgresConversationRepository {
    pool: DbPool,
}

impl PostgresConversationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepository for PostgresConversationRepository {
    async fn get_or_create(&self, pair: ConversationPair) -> Result<Conversation, RepoError> {
        queries::get_or_create(&self.pool, pair).await
    }

    async fn find_by_id(&self, id: ConversationId) -> Result<Option<Conversation>, RepoError> {
        queries::find_by_id(&self.pool, id).await
    }

    async fn append(
        &self,
        conversation_id: ConversationId,
        new_message: NewMessage,
    ) -> Result<Message, RepoError> {
        queries::append(&self.pool, conversation_id, new_message).await
    }

    async fn list_for_user(
        &self,
        user_id: UserId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<ConversationInboxPage, RepoError> {
        queries::list_for_user(&self.pool, user_id, cursor, limit).await
    }

    async fn messages(
        &self,
        conversation_id: ConversationId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<MessagePage, RepoError> {
        queries::messages(&self.pool, conversation_id, cursor, limit).await
    }

    async fn mark_read(
        &self,
        conversation_id: ConversationId,
        user_id: UserId,
    ) -> Result<(), RepoError> {
        queries::mark_read(&self.pool, conversation_id, user_id).await
    }

    async fn unread_exists(&self, user_id: UserId) -> Result<bool, RepoError> {
        queries::unread_exists(&self.pool, user_id).await
    }

    async fn list_offer_inquirers(
        &self,
        offer_id: OfferId,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<OfferInquirersPage, RepoError> {
        queries::list_offer_inquirers(&self.pool, offer_id, cursor, limit).await
    }
}
