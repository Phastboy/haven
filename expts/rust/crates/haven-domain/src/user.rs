//! User entity and participant identifier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::account::AccountId;

/// Newtype wrapping a UUID that identifies a User (platform participant).
/// Cannot be confused with `AccountId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Generates a new random `UserId`.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying raw UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Platform participant (authz + platform context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier of the participant.
    pub id: UserId,
    /// Associated account identifier.
    pub account_id: AccountId,
    /// Timestamp when this participant was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this participant was last updated.
    pub updated_at: DateTime<Utc>,
}
