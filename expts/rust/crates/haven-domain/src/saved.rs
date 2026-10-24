//! Saved offer and followed person records.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{offer::OfferId, user::User};

/// A saved offer in a person's private collection.
#[derive(Debug, Clone)]
pub struct SavedOffer {
    /// Offer being saved.
    pub offer_id: OfferId,
    /// Current offer details, when the offer still exists and is public.
    pub offer: Option<crate::offer::Offer>,
    /// When the person saved the offer.
    pub created_at: DateTime<Utc>,
    /// Stable pagination id.
    pub id: Uuid,
}

/// A person followed by the signed-in person.
#[derive(Debug, Clone)]
pub struct FollowedPerson {
    /// Followed person identity.
    pub person: User,
    /// When the follow was created.
    pub created_at: DateTime<Utc>,
    /// Stable pagination id.
    pub id: Uuid,
}

/// Cursor for a person's saved offers or following list.
#[derive(Debug, Clone, Copy)]
pub struct CollectionCursor {
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Tie-breaking row id.
    pub id: Uuid,
}

impl CollectionCursor {
    /// Creates a cursor.
    #[must_use]
    pub fn new(created_at: DateTime<Utc>, id: Uuid) -> Self {
        Self { created_at, id }
    }

    /// Encodes the cursor for a URL.
    #[must_use]
    pub fn encode(self) -> String {
        format!("{}~{}", self.created_at.timestamp_micros(), self.id)
    }

    /// Decodes a URL cursor.
    #[must_use]
    pub fn decode(raw: &str) -> Option<Self> {
        let (time, id) = raw.split_once('~')?;
        let micros = time.parse::<i64>().ok()?;
        let created_at = DateTime::from_timestamp_micros(micros)?;
        Some(Self {
            created_at,
            id: Uuid::parse_str(id).ok()?,
        })
    }
}

/// A keyset page of saved offers.
pub struct SavedOfferPage {
    /// Page entries.
    pub items: Vec<SavedOffer>,
    /// Next cursor, if another page exists.
    pub next_cursor: Option<String>,
}

/// A keyset page of followed people.
pub struct FollowPage {
    /// Page entries.
    pub items: Vec<FollowedPerson>,
    /// Next cursor, if another page exists.
    pub next_cursor: Option<String>,
}
