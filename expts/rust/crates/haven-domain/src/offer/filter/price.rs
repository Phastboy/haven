//! Validated price and currency criterion.

use crate::{
    error::FilterValidationError,
    offer::{CurrencyCode, Price},
};
use serde::{Deserialize, Serialize};

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
