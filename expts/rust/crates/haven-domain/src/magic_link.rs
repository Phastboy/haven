use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::account::Email;
use crate::session::{HashedToken, PlaintextToken};

/// Strongly typed identifier for a magic link record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MagicLinkId(Uuid);

impl MagicLinkId {
    /// Generates a new random `MagicLinkId`.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs a `MagicLinkId` from an existing UUID.
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

impl From<Uuid> for MagicLinkId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for MagicLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A single-use credential delivered to a user's email address.
/// Conceptually identical to a `PlaintextToken`, alias added for domain clarity.
pub type VerificationToken = PlaintextToken;

/// A magic link record tracking the lifecycle of an authentication attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLink {
    pub id: MagicLinkId,
    pub email: Email,
    pub token_hash: HashedToken,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl MagicLink {
    /// Is this magic link valid (not expired, not used) at the given time?
    #[must_use]
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        self.used_at.is_none() && now < self.expires_at
    }
}
