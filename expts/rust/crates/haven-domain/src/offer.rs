//! Offer aggregate, identifiers, commands, and submodule assembly.

pub mod currency;
pub mod cursor;
pub mod filter;
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
pub use filter::{PriceFilter, PublicCriteria, PublicCriterion, Scope, SearchQuery};
pub use price::Price;
pub use slug::{MAX_SLUG_CHARS, MIN_SLUG_CHARS, OfferSlug, RESERVED_SLUGS};
pub use validation::{
    MAX_DESCRIPTION_CHARS, MAX_TITLE_CHARS, MIN_DESCRIPTION_CHARS, MIN_TITLE_CHARS,
    validate_description, validate_price_and_currency, validate_title,
};

use crate::error::OfferValidationError;
pub use crate::user::{DisplayName, ProfileHandle, UserId};

/// Query parameters for listing public offers on the feed.
#[derive(Debug, Clone, Default)]
pub struct PublicFeedQuery {
    scope: Scope,
    /// Criteria for filtering offers by title, price, and currency.
    pub criteria: PublicCriteria,
    /// Optional keyset cursor to paginate after.
    pub cursor: Option<OfferCursor>,
    /// Maximum number of items to return in a single page.
    pub limit: usize,
}

impl PublicFeedQuery {
    /// Returns the collection permitted by this public-only query type.
    ///
    /// Personal scope variants cannot be passed to this API and silently ignored.
    #[must_use]
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// Constructs a new public feed query.
    #[must_use]
    pub fn new(criteria: PublicCriteria, cursor: Option<OfferCursor>, limit: usize) -> Self {
        Self {
            scope: Scope::Public,
            criteria,
            cursor,
            limit,
        }
    }
}

/// Public offer presentation containing core offer details and maker identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicOffer {
    /// Associated core offer.
    pub offer: Offer,
    /// Public display name of the maker.
    pub maker_display_name: DisplayName,
    /// Public unique handle of the maker.
    pub maker_handle: ProfileHandle,
}

/// Paginated page of public offers with maker details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicOfferPage {
    /// List of public offers in this page.
    pub items: Vec<PublicOffer>,
    /// Opaque next cursor token for pagination, if more items exist.
    pub next_cursor: Option<String>,
}

/// Newtype wrapping a UUID that identifies an Offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OfferId(Uuid);

impl OfferId {
    /// Generates a new random `OfferId`.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs an `OfferId` from an existing UUID.
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

impl From<Uuid> for OfferId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
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
    /// Total count of uploaded images (0..=5).
    pub image_count: u8,
    /// Uploaded images ordered by position index.
    pub images: Vec<crate::image::OfferImage>,
    /// Timestamp when this offer was published.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this offer was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Offer {
    /// Returns true if the offer has at least 1 image and is publicly discoverable.
    #[must_use]
    pub fn is_public(&self) -> bool {
        self.image_count >= 1
    }

    /// Returns the cover image (position 1) if present.
    #[must_use]
    pub fn cover_image(&self) -> Option<&crate::image::OfferImage> {
        self.images.iter().find(|img| img.position.is_cover())
    }
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
    /// Returns [`OfferValidationError`] if title, description, or price/currency are invalid.
    pub fn validate(&self) -> Result<(), OfferValidationError> {
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
    /// Returns [`OfferValidationError`] if provided title or description violate length constraints.
    pub fn validate(&self) -> Result<(), OfferValidationError> {
        if let Some(ref title) = self.title {
            validate_title(title)?;
        }
        if let Some(ref desc) = self.description {
            validate_description(desc)?;
        }
        Ok(())
    }
}
