//! ISO 4217 currency code value object and supported currencies whitelist.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::OfferValidationError;

/// Currencies permitted by Haven.
pub const SUPPORTED_CURRENCIES: &[&str] = &["NGN", "USD", "EUR", "GBP", "CAD", "AUD", "KES", "GHS"];

/// A 3-character ISO 4217 currency code whitelisted for Haven.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum CurrencyCode {
    Ngn,
    Usd,
    Eur,
    Gbp,
    Cad,
    Aud,
    Kes,
    Ghs,
}

impl TryFrom<String> for CurrencyCode {
    type Error = OfferValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl<'a> TryFrom<&'a str> for CurrencyCode {
    type Error = OfferValidationError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<CurrencyCode> for String {
    fn from(value: CurrencyCode) -> Self {
        value.as_str().to_owned()
    }
}

impl CurrencyCode {
    /// Returns the platform default currency code (`NGN`).
    #[must_use]
    pub fn default_code() -> Self {
        Self::Ngn
    }

    /// Parses and validates a raw currency code against [`SUPPORTED_CURRENCIES`].
    ///
    /// # Errors
    ///
    /// Returns [`OfferValidationError::InvalidCurrencyCode`] if `raw` is not supported.
    pub fn parse(raw: &str) -> Result<Self, OfferValidationError> {
        match raw.trim().to_uppercase().as_str() {
            "NGN" => Ok(Self::Ngn),
            "USD" => Ok(Self::Usd),
            "EUR" => Ok(Self::Eur),
            "GBP" => Ok(Self::Gbp),
            "CAD" => Ok(Self::Cad),
            "AUD" => Ok(Self::Aud),
            "KES" => Ok(Self::Kes),
            "GHS" => Ok(Self::Ghs),
            _ => Err(OfferValidationError::InvalidCurrencyCode),
        }
    }

    /// Returns the uppercase 3-letter currency code as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ngn => "NGN",
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
            Self::Cad => "CAD",
            Self::Aud => "AUD",
            Self::Kes => "KES",
            Self::Ghs => "GHS",
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CurrencyCode {
    type Err = OfferValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
