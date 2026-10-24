//! Offer filter criteria for searching and narrowing public offers.

use serde::{Deserialize, Serialize};

use crate::error::FilterValidationError;
use crate::offer::currency::CurrencyCode;
use crate::offer::price::Price;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;

/// Validated search query string for matching offer titles.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// Maximum character length for a title search query.
    pub const MAX_CHARS: usize = 100;

    /// Validates and constructs a search query.
    ///
    /// # Errors
    ///
    /// Returns [`FilterValidationError::BlankSearchQuery`] if `raw` is empty after trimming,
    /// or [`FilterValidationError::SearchQueryTooLong`] if it exceeds [`MAX_CHARS`].
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, FilterValidationError> {
        let trimmed = raw.as_ref().trim();
        if trimmed.is_empty() {
            return Err(FilterValidationError::BlankSearchQuery);
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(FilterValidationError::SearchQueryTooLong);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the underlying raw query string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Filter criteria to narrow offers by price range and currency.
///
/// A price filter is strictly paired with an ISO 4217 currency code.
/// A price of zero denotes a Free offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceFilter {
    /// ISO 4217 currency code paired with the price filter.
    pub currency: CurrencyCode,
    /// Minimum price (inclusive), in minor currency units.
    pub min: Option<Price>,
    /// Maximum price (inclusive), in minor currency units.
    pub max: Option<Price>,
}

impl PriceFilter {
    /// Constructs a validated price filter.
    ///
    /// At least one of `min` or `max` must be provided.
    ///
    /// # Errors
    ///
    /// Returns [`FilterValidationError::InvalidPriceRange`] if `min > max`.
    /// Returns [`FilterValidationError::EmptyPriceFilter`] if both `min` and `max` are `None`.
    pub fn new(
        currency: CurrencyCode,
        min: Option<Price>,
        max: Option<Price>,
    ) -> Result<Self, FilterValidationError> {
        if min.is_none() && max.is_none() {
            return Err(FilterValidationError::EmptyPriceFilter);
        }
        if let (Some(min_p), Some(max_p)) = (min, max)
            && min_p.as_i32() > max_p.as_i32()
        {
            return Err(FilterValidationError::InvalidPriceRange);
        }
        Ok(Self { currency, min, max })
    }

    /// Constructs a filter specifically for Free offers (price == 0).
    #[must_use]
    pub fn free(currency: CurrencyCode) -> Self {
        Self {
            currency,
            min: Some(Price::ZERO),
            max: Some(Price::ZERO),
        }
    }

    /// Returns true if this filter targets exclusively Free offers (max is 0).
    #[must_use]
    pub fn is_free_only(&self) -> bool {
        self.max.is_some_and(|p| p.as_i32() == 0)
    }
}

/// Filter criteria for querying public offers.
///
/// All fields are [`Option`], permitting unconstrained feeds or arbitrary combinations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferFilter {
    /// Optional title search query.
    pub title: Option<SearchQuery>,
    /// Optional price and currency filter.
    pub price: Option<PriceFilter>,
}

impl OfferFilter {
    /// Returns `true` if no filter criteria are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.price.is_none()
    }
}
