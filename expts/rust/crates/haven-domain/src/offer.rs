use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DomainError;

/// Newtype wrapping a UUID that identifies an Offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OfferId(pub Uuid);

impl OfferId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

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

/// Newtype wrapping a UUID that identifies a User (platform participant).
/// Cannot be confused with `AccountId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A non-negative price in minor currency units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Price(i32);

impl Price {
    pub const ZERO: Self = Self(0);

    pub fn new(value: i32) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::InvalidPrice);
        }
        Ok(Self(value))
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

/// A 3-character ISO 4217 currency code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn default_code() -> Self {
        Self("NGN".to_string())
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let code = raw.trim().to_uppercase();
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code))
        } else {
            Err(DomainError::InvalidCurrencyCode)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Core offer entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: OfferId,
    pub user_id: UserId,
    pub title: String,
    pub description: Option<String>,
    pub price: Price,
    pub currency: CurrencyCode,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Offer {
}

/// Data needed to create a new offer.
#[derive(Debug, Clone)]
pub struct CreateOffer {
    pub title: String,
    pub description: Option<String>,
    pub price: Price,
    pub currency: CurrencyCode,
}

impl CreateOffer {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::BlankTitle);
        }
        if self.title.len() > 255 {
            return Err(DomainError::TitleTooLong);
        }
        Ok(())
    }
}

/// Data needed to update an existing offer.
#[derive(Debug, Clone)]
pub struct UpdateOffer {
    pub title: String,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub currency: Option<CurrencyCode>,
}

impl UpdateOffer {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::BlankTitle);
        }
        if self.title.len() > 255 {
            return Err(DomainError::TitleTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_must_be_non_negative() {
        assert!(Price::new(-1).is_err());
        assert!(Price::new(0).is_ok());
        assert!(Price::new(100).is_ok());
    }

    #[test]
    fn currency_code_must_be_3_chars() {
        assert!(CurrencyCode::parse("usd").is_ok()); // gets uppercased
        assert!(CurrencyCode::parse("USD").is_ok());
        assert!(CurrencyCode::parse("US").is_err());
        assert!(CurrencyCode::parse("USDA").is_err());
        assert!(CurrencyCode::parse("US1").is_err());
    }

    #[test]
    fn create_offer_validates_title() {
        let mut co = CreateOffer {
            title: "  ".into(),
            description: None,
            price: Price::ZERO,
            currency: CurrencyCode::default_code(),
        };
        assert!(matches!(co.validate(), Err(DomainError::BlankTitle)));

        co.title = "A".repeat(256);
        assert!(matches!(co.validate(), Err(DomainError::TitleTooLong)));
    }
}
