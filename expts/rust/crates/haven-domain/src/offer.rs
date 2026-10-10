//! Offer aggregate, identifiers, commands, and submodule assembly.

pub mod currency;
pub mod cursor;
pub mod price;
pub mod slug;
pub mod validation;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use currency::{CurrencyCode, SUPPORTED_CURRENCIES};
pub use cursor::{OfferCursor, OfferPage};
pub use price::Price;
pub use slug::{MAX_SLUG_CHARS, MIN_SLUG_CHARS, OfferSlug, RESERVED_SLUGS};
pub use validation::{
    MAX_DESCRIPTION_CHARS, MAX_TITLE_CHARS, MIN_DESCRIPTION_CHARS, MIN_TITLE_CHARS,
    validate_description, validate_price_and_currency, validate_title,
};

use crate::DomainError;
pub use crate::user::UserId;

/// Newtype wrapping a UUID that identifies an Offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OfferId(pub Uuid);

impl OfferId {
    /// Generates a new random `OfferId`.
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

impl Default for OfferId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OfferId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Core offer entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    /// Unique identifier of the offer.
    pub id: OfferId,
    /// Participant identifier who owns and published the offer.
    pub user_id: UserId,
    /// Public URL-safe slug.
    pub slug: OfferSlug,
    /// Display title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Price value in minor currency units.
    pub price: Price,
    /// ISO 4217 currency code.
    pub currency: CurrencyCode,
    /// Timestamp when this offer was published.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this offer was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Data needed to create a new offer.
#[derive(Debug, Clone)]
pub struct CreateOffer {
    /// Proposed display title.
    pub title: String,
    /// Proposed detailed description.
    pub description: String,
    /// Proposed price in minor currency units.
    pub price: Price,
    /// Proposed ISO 4217 currency code.
    pub currency: CurrencyCode,
}

impl CreateOffer {
    /// Validates all fields against domain bounds and business rules.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] if title, description, or price/currency are invalid.
    pub fn validate(&self) -> Result<(), DomainError> {
        validate_title(&self.title)?;
        validate_description(&self.description)?;
        validate_price_and_currency(self.price, Some(&self.currency))?;
        Ok(())
    }
}

/// Data needed to update an existing offer.
#[derive(Debug, Clone)]
pub struct UpdateOffer {
    /// Updated title, or `None` to retain current.
    pub title: Option<String>,
    /// Updated description, or `None` to retain current.
    pub description: Option<String>,
    /// Updated price, or `None` to retain current.
    pub price: Option<Price>,
    /// Updated currency, or `None` to retain current.
    pub currency: Option<CurrencyCode>,
}

impl UpdateOffer {
    /// Validates provided fields against domain bounds.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] if provided title or description violate length constraints.
    pub fn validate(&self) -> Result<(), DomainError> {
        if let Some(ref title) = self.title {
            validate_title(title)?;
        }
        if let Some(ref desc) = self.description {
            validate_description(desc)?;
        }
        Ok(())
    }
}
