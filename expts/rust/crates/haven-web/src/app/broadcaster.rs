//! In-process broadcast port and implementation for conversation messages.

#![allow(
    clippy::unwrap_used,
    clippy::let_underscore_must_use,
    reason = "In-memory broadcast channel lock and send"
)]

use haven_domain::conversation::{ConversationId, Message};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub trait MessageBroadcaster: Send + Sync {
    /// Broadcast a new message to all active listeners for this conversation.
    fn broadcast(&self, message: &Message);

    /// Subscribe to events for a specific conversation.
    fn subscribe(&self, conversation_id: ConversationId) -> broadcast::Receiver<Message>;
}

#[derive(Clone, Default)]
pub struct InProcessBroadcaster {
    channels: Arc<Mutex<HashMap<ConversationId, broadcast::Sender<Message>>>>,
}

impl InProcessBroadcaster {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MessageBroadcaster for InProcessBroadcaster {
    fn broadcast(&self, message: &Message) {
        let channels = self.channels.lock().unwrap();
        if let Some(tx) = channels.get(&message.conversation_id) {
            // Ignore Err if no active receivers currently listening
            let _ = tx.send(message.clone());
        }
    }

    fn subscribe(&self, conversation_id: ConversationId) -> broadcast::Receiver<Message> {
        let mut channels = self.channels.lock().unwrap();
        let tx = channels.entry(conversation_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(100);
            tx
        });
        tx.subscribe()
    }
}
