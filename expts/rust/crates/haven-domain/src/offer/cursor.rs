//! Keyset pagination cursor and paginated page structure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Offer, OfferId};
use crate::error::CursorError;

/// Keyset cursor for deterministic, constant-time feed pagination on `(created_at, id)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferCursor {
    /// Timestamp of the offer used as reference.
    pub created_at: DateTime<Utc>,
    /// Identifier of the offer used as tiebreaker.
    pub id: OfferId,
}

impl OfferCursor {
    /// Creates a new keyset pagination cursor.
    #[must_use]
    pub fn new(created_at: DateTime<Utc>, id: OfferId) -> Self {
        Self { created_at, id }
    }

    /// Encodes this cursor into a URL-safe hex string.
    #[must_use]
    pub fn encode(&self) -> String {
        let raw = format!(
            "{}_{}",
            self.created_at.timestamp_micros(),
            self.id.as_uuid()
        );
        hex::encode(raw.as_bytes())
    }

    /// Decodes an opaque hex string back into an [`OfferCursor`].
    ///
    /// # Errors
    ///
    /// Returns [`CursorError::InvalidCursor`] if the string cannot be decoded or parsed.
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
        let uuid = Uuid::parse_str(uuid_str).map_err(|_| CursorError::InvalidCursor)?;
        let created_at =
            DateTime::from_timestamp_micros(micros).ok_or(CursorError::InvalidCursor)?;

        Ok(Self {
            created_at,
            id: OfferId::from_uuid(uuid),
        })
    }
}

/// A page of offers returned from a keyset query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferPage {
    /// List of offers in this page.
    pub items: Vec<Offer>,
    /// Opaque next cursor token for pagination, if more items exist.
    pub next_cursor: Option<String>,
}
