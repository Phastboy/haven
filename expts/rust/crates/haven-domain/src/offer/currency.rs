//! ISO 4217 currency code value object and supported currencies whitelist.

use serde::{Deserialize, Serialize};

use crate::DomainError;

/// Currencies permitted by Haven.
pub const SUPPORTED_CURRENCIES: &[&str] = &["NGN", "USD", "EUR", "GBP", "CAD", "AUD", "KES", "GHS"];

/// A 3-character ISO 4217 currency code whitelisted for Haven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Returns the platform default currency code (`NGN`).
    #[must_use]
    pub fn default_code() -> Self {
        Self("NGN".to_string())
    }

    /// Parses and validates a raw currency code against [`SUPPORTED_CURRENCIES`].
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCurrencyCode`] if `raw` is not supported.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let code = raw.trim().to_uppercase();
        if SUPPORTED_CURRENCIES.contains(&code.as_str()) {
            Ok(Self(code))
        } else {
            Err(DomainError::InvalidCurrencyCode)
        }
    }

    /// Returns the uppercase 3-letter currency code as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
