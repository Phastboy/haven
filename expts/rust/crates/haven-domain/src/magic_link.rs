use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::account::Email;
use crate::session::{HashedToken, PlaintextToken};

/// A single-use credential delivered to a user's email address.
/// Conceptually identical to a PlaintextToken, alias added for domain clarity.
pub type VerificationToken = PlaintextToken;

/// A magic link record tracking the lifecycle of an authentication attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLink {
    pub id: Uuid,
    pub email: Email,
    pub token_hash: HashedToken,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl MagicLink {
    /// Is this magic link valid (not expired, not used)?
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && Utc::now() < self.expires_at
    }
}
