//! Non-negative price value object.

use serde::{Deserialize, Serialize};

use crate::error::OfferValidationError;

/// A non-negative price in minor currency units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct Price(i32);

impl TryFrom<i32> for Price {
    type Error = OfferValidationError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Price> for i32 {
    fn from(p: Price) -> Self {
        p.0
    }
}

impl Price {
    /// Zero price constant.
    pub const ZERO: Self = Self(0);

    /// Constructs a new `Price` after validating non-negativity.
    ///
    /// # Errors
    ///
    /// Returns [`OfferValidationError::InvalidPrice`] if `value` is negative.
    pub fn new(value: i32) -> Result<Self, OfferValidationError> {
        if value < 0 {
            return Err(OfferValidationError::InvalidPrice);
        }
        Ok(Self(value))
    }

    /// Returns the raw integer value in minor units.
    #[must_use]
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}
