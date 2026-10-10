//! Non-negative price value object.

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// A non-negative price in minor currency units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price(i32);

impl Price {
    /// Zero price constant.
    pub const ZERO: Self = Self(0);

    /// Constructs a new `Price` after validating non-negativity.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidPrice`] if `value` is negative.
    pub fn new(value: i32) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::InvalidPrice);
        }
        Ok(Self(value))
    }

    /// Returns the raw integer value in minor units.
    #[must_use]
    pub fn as_i32(&self) -> i32 {
        self.0
    }
}
