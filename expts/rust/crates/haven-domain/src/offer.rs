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

/// Minimum character count for an offer title.
pub const MIN_TITLE_CHARS: usize = 3;

/// Maximum character count for an offer title.
pub const MAX_TITLE_CHARS: usize = 100;

/// Minimum character count for an offer description.
pub const MIN_DESCRIPTION_CHARS: usize = 10;

/// Maximum character count for an offer description.
pub const MAX_DESCRIPTION_CHARS: usize = 2000;

/// Currencies permitted by Haven.
pub const SUPPORTED_CURRENCIES: &[&str] = &["NGN", "USD", "EUR", "GBP", "CAD", "AUD", "KES", "GHS"];

/// Validates that an offer title meets character length constraints.
pub fn validate_title(title: &str) -> Result<(), DomainError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankTitle);
    }
    let count = trimmed.chars().count();
    if count < MIN_TITLE_CHARS {
        return Err(DomainError::TitleTooShort);
    }
    if count > MAX_TITLE_CHARS {
        return Err(DomainError::TitleTooLong);
    }
    Ok(())
}

/// Validates that an offer description meets character length constraints.
pub fn validate_description(description: &str) -> Result<(), DomainError> {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankDescription);
    }
    let count = trimmed.chars().count();
    if count < MIN_DESCRIPTION_CHARS {
        return Err(DomainError::DescriptionTooShort);
    }
    if count > MAX_DESCRIPTION_CHARS {
        return Err(DomainError::DescriptionTooLong);
    }
    Ok(())
}

/// Validates that an offer price and currency combination is valid.
/// When price is greater than zero, currency must be explicitly provided.
pub fn validate_price_and_currency(
    price: Price,
    currency: Option<&CurrencyCode>,
) -> Result<(), DomainError> {
    if price.as_i32() > 0 && currency.is_none() {
        return Err(DomainError::CurrencyRequired);
    }
    Ok(())
}

/// A 3-character ISO 4217 currency code whitelisted for Haven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn default_code() -> Self {
        Self("NGN".to_string())
    }

    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let code = raw.trim().to_uppercase();
        if SUPPORTED_CURRENCIES.contains(&code.as_str()) {
            Ok(Self(code))
        } else {
            Err(DomainError::InvalidCurrencyCode)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Minimum character count for an offer slug.
pub const MIN_SLUG_CHARS: usize = 3;

/// Maximum character count for an offer slug.
pub const MAX_SLUG_CHARS: usize = 120;

/// Slugs reserved by the routing structure that cannot identify an offer.
pub const RESERVED_SLUGS: &[&str] = &["manage", "new"];

/// A URL-safe unique slug for public offer lookup.
/// Conforms to lowercase alphanumeric segments separated by hyphens (e.g. `vintage-chair-a1b2c3d4`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OfferSlug(String);

impl OfferSlug {
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let trimmed = s.trim();
        let len = trimmed.len();
        if !(MIN_SLUG_CHARS..=MAX_SLUG_CHARS).contains(&len) {
            return Err(DomainError::InvalidSlug(s.to_string()));
        }

        if RESERVED_SLUGS.contains(&trimmed) {
            return Err(DomainError::InvalidSlug(s.to_string()));
        }

        let mut prev_hyphen = false;
        for (i, c) in trimmed.chars().enumerate() {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                prev_hyphen = false;
            } else if c == '-' {
                if i == 0 || prev_hyphen {
                    return Err(DomainError::InvalidSlug(s.to_string()));
                }
                prev_hyphen = true;
            } else {
                return Err(DomainError::InvalidSlug(s.to_string()));
            }
        }

        if prev_hyphen {
            return Err(DomainError::InvalidSlug(s.to_string()));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Derives a clean URL-safe slug from a title and a unique suffix (e.g. offer id prefix).
    pub fn from_title_and_suffix(title: &str, suffix: &str) -> Result<Self, DomainError> {
        let mut base = String::new();
        let mut last_was_hyphen = false;
        for c in title.trim().chars() {
            if c.is_ascii_alphanumeric() {
                base.push(c.to_ascii_lowercase());
                last_was_hyphen = false;
            } else if !last_was_hyphen && !base.is_empty() {
                base.push('-');
                last_was_hyphen = true;
            }
        }
        let mut base_clean = base.trim_end_matches('-').to_string();
        if base_clean.is_empty() {
            base_clean = "offer".to_string();
        }
        if base_clean.len() > 80 {
            let mut truncated = String::new();
            for c in base_clean.chars().take(80) {
                truncated.push(c);
            }
            base_clean = truncated.trim_end_matches('-').to_string();
            if base_clean.is_empty() {
                base_clean = "offer".to_string();
            }
        }

        let mut suffix_clean = String::new();
        for c in suffix
            .to_ascii_lowercase()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .take(8)
        {
            suffix_clean.push(c);
        }
        let suffix_val = if suffix_clean.is_empty() {
            "00000000"
        } else {
            &suffix_clean
        };

        let combined = format!("{base_clean}-{suffix_val}");
        Self::parse(&combined)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OfferSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Keyset cursor for deterministic, constant-time feed pagination on `(created_at, id)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferCursor {
    pub created_at: DateTime<Utc>,
    pub id: OfferId,
}

impl OfferCursor {
    pub fn new(created_at: DateTime<Utc>, id: OfferId) -> Self {
        Self { created_at, id }
    }

    pub fn encode(&self) -> String {
        let raw = format!(
            "{}_{}",
            self.created_at.timestamp_micros(),
            self.id.as_uuid()
        );
        hex::encode(raw.as_bytes())
    }

    pub fn decode(encoded: &str) -> Result<Self, DomainError> {
        let bytes = hex::decode(encoded).map_err(|_| DomainError::InvalidCursor)?;
        let raw = String::from_utf8(bytes).map_err(|_| DomainError::InvalidCursor)?;
        let mut parts = raw.split('_');
        let micros_str = parts.next().ok_or(DomainError::InvalidCursor)?;
        let uuid_str = parts.next().ok_or(DomainError::InvalidCursor)?;
        if parts.next().is_some() {
            return Err(DomainError::InvalidCursor);
        }
        let micros: i64 = micros_str.parse().map_err(|_| DomainError::InvalidCursor)?;
        let uuid = Uuid::parse_str(uuid_str).map_err(|_| DomainError::InvalidCursor)?;
        let created_at =
            DateTime::from_timestamp_micros(micros).ok_or(DomainError::InvalidCursor)?;

        Ok(Self {
            created_at,
            id: OfferId(uuid),
        })
    }
}

/// A page of offers returned from a keyset query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferPage {
    pub items: Vec<Offer>,
    pub next_cursor: Option<String>,
}

/// Core offer entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: OfferId,
    pub user_id: UserId,
    pub slug: OfferSlug,
    pub title: String,
    pub description: String,
    pub price: Price,
    pub currency: CurrencyCode,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Offer {}

/// Data needed to create a new offer.
#[derive(Debug, Clone)]
pub struct CreateOffer {
    pub title: String,
    pub description: String,
    pub price: Price,
    pub currency: CurrencyCode,
}

impl CreateOffer {
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
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub currency: Option<CurrencyCode>,
}

impl UpdateOffer {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn price_must_be_non_negative() {
        assert!(Price::new(-1).is_err());
        assert!(Price::new(0).is_ok());
        assert!(Price::new(100).is_ok());
    }

    #[test]
    fn currency_code_accepts_whitelisted_currencies() {
        assert!(CurrencyCode::parse("usd").is_ok()); // case-insensitive
        assert!(CurrencyCode::parse("USD").is_ok());
        assert!(CurrencyCode::parse("ngn").is_ok());
        assert!(CurrencyCode::parse("NGN").is_ok());
        assert!(CurrencyCode::parse("EUR").is_ok());
        assert!(CurrencyCode::parse("GBP").is_ok());
        assert!(CurrencyCode::parse("CAD").is_ok());
        assert!(CurrencyCode::parse("AUD").is_ok());
        assert!(CurrencyCode::parse("KES").is_ok());
        assert!(CurrencyCode::parse("GHS").is_ok());
    }

    #[test]
    fn currency_code_rejects_unsupported_or_garbage_codes() {
        assert!(CurrencyCode::parse("US").is_err());
        assert!(CurrencyCode::parse("USDA").is_err());
        assert!(CurrencyCode::parse("US1").is_err());
        assert!(CurrencyCode::parse("ZZZ").is_err()); // non-whitelisted 3-letter code
        assert!(CurrencyCode::parse("XYZ").is_err());
        assert!(CurrencyCode::parse("").is_err());
        assert!(CurrencyCode::parse("   ").is_err());
    }

    #[test]
    fn validate_title_enforces_character_bounds() {
        assert!(matches!(validate_title(""), Err(DomainError::BlankTitle)));
        assert!(matches!(
            validate_title("   "),
            Err(DomainError::BlankTitle)
        ));
        assert!(matches!(
            validate_title("ab"),
            Err(DomainError::TitleTooShort)
        ));
        assert!(validate_title("abc").is_ok());

        let exactly_100_chars = "a".repeat(100);
        assert!(validate_title(&exactly_100_chars).is_ok());

        let over_100_chars = "a".repeat(101);
        assert!(matches!(
            validate_title(&over_100_chars),
            Err(DomainError::TitleTooLong)
        ));

        // Multi-byte Unicode: 3 emojis = 3 characters (12 bytes), should be valid
        let emoji_title = "🎉✨🚀";
        assert_eq!(emoji_title.chars().count(), 3);
        assert!(validate_title(emoji_title).is_ok());
    }

    #[test]
    fn validate_description_enforces_character_bounds() {
        assert!(matches!(
            validate_description(""),
            Err(DomainError::BlankDescription)
        ));
        assert!(matches!(
            validate_description("   "),
            Err(DomainError::BlankDescription)
        ));
        assert!(matches!(
            validate_description("123456789"),
            Err(DomainError::DescriptionTooShort)
        ));
        assert!(validate_description("1234567890").is_ok());

        let exactly_2000_chars = "x".repeat(2000);
        assert!(validate_description(&exactly_2000_chars).is_ok());

        let over_2000_chars = "x".repeat(2001);
        assert!(matches!(
            validate_description(&over_2000_chars),
            Err(DomainError::DescriptionTooLong)
        ));

        // Multi-byte Unicode
        let unicode_desc = "Café au lait 🥐!";
        assert!(unicode_desc.chars().count() >= 10);
        assert!(validate_description(unicode_desc).is_ok());
    }

    #[test]
    fn create_offer_validates_title_and_description() {
        let mut co = CreateOffer {
            title: "Valid Title".into(),
            description: "Valid description here".into(),
            price: Price::ZERO,
            currency: CurrencyCode::default_code(),
        };
        assert!(co.validate().is_ok());

        co.title = "  ".into();
        assert!(matches!(co.validate(), Err(DomainError::BlankTitle)));

        co.title = "Valid Title".into();
        co.description = "short".into();
        assert!(matches!(
            co.validate(),
            Err(DomainError::DescriptionTooShort)
        ));

        co.description = "This is a valid offer description.".into();
        assert!(co.validate().is_ok());
    }

    #[test]
    fn update_offer_validates_partial_updates() {
        let uo = UpdateOffer {
            title: None,
            description: None,
            price: None,
            currency: None,
        };
        assert!(uo.validate().is_ok());

        let uo_invalid_title = UpdateOffer {
            title: Some("ab".into()),
            description: None,
            price: None,
            currency: None,
        };
        assert!(matches!(
            uo_invalid_title.validate(),
            Err(DomainError::TitleTooShort)
        ));

        let uo_invalid_desc = UpdateOffer {
            title: None,
            description: Some("short".into()),
            price: None,
            currency: None,
        };
        assert!(matches!(
            uo_invalid_desc.validate(),
            Err(DomainError::DescriptionTooShort)
        ));

        let uo_overlong_title = UpdateOffer {
            title: Some("a".repeat(101)),
            description: None,
            price: None,
            currency: None,
        };
        assert!(matches!(
            uo_overlong_title.validate(),
            Err(DomainError::TitleTooLong)
        ));

        let uo_overlong_desc = UpdateOffer {
            title: None,
            description: Some("x".repeat(2001)),
            price: None,
            currency: None,
        };
        assert!(matches!(
            uo_overlong_desc.validate(),
            Err(DomainError::DescriptionTooLong)
        ));
    }

    #[test]
    fn create_offer_rejects_overlong_inputs() {
        let mut co = CreateOffer {
            title: "a".repeat(101),
            description: "A valid description for testing.".into(),
            price: Price::ZERO,
            currency: CurrencyCode::default_code(),
        };
        assert!(matches!(co.validate(), Err(DomainError::TitleTooLong)));

        co.title = "Valid Title".into();
        co.description = "x".repeat(2001);
        assert!(matches!(
            co.validate(),
            Err(DomainError::DescriptionTooLong)
        ));
    }

    #[test]
    fn all_supported_currencies_parse_and_roundtrip() {
        for &code in SUPPORTED_CURRENCIES {
            let parsed = CurrencyCode::parse(code).expect("supported currency should parse");
            assert_eq!(parsed.as_str(), code);

            let lower = code.to_lowercase();
            let parsed_lower =
                CurrencyCode::parse(&lower).expect("lowercase currency should parse");
            assert_eq!(parsed_lower.as_str(), code);
        }
    }

    #[test]
    fn unicode_multibyte_characters_counted_by_chars_not_bytes() {
        // 100 emoji characters = 400 bytes, but exactly 100 characters
        let title_100_emojis = "🎉".repeat(100);
        assert_eq!(title_100_emojis.chars().count(), 100);
        assert_eq!(title_100_emojis.len(), 400);
        assert!(validate_title(&title_100_emojis).is_ok());

        // 101 emoji characters = 101 characters -> Too long
        let title_101_emojis = "🎉".repeat(101);
        assert!(matches!(
            validate_title(&title_101_emojis),
            Err(DomainError::TitleTooLong)
        ));

        // 2000 emoji characters = 8000 bytes, but exactly 2000 characters
        let desc_2000_emojis = "✨".repeat(2000);
        assert_eq!(desc_2000_emojis.chars().count(), 2000);
        assert!(validate_description(&desc_2000_emojis).is_ok());

        // 2001 emoji characters -> Too long
        let desc_2001_emojis = "✨".repeat(2001);
        assert!(matches!(
            validate_description(&desc_2001_emojis),
            Err(DomainError::DescriptionTooLong)
        ));
    }

    #[test]
    fn price_and_currency_validation() {
        let zero = Price::ZERO;
        let positive = Price::new(500).unwrap();
        let usd = CurrencyCode::parse("USD").unwrap();

        assert!(validate_price_and_currency(zero, None).is_ok());
        assert!(validate_price_and_currency(zero, Some(&usd)).is_ok());
        assert_eq!(
            validate_price_and_currency(positive, None),
            Err(DomainError::CurrencyRequired)
        );
        assert!(validate_price_and_currency(positive, Some(&usd)).is_ok());
    }

    #[test]
    fn offer_slug_validates_format() {
        assert!(OfferSlug::parse("vintage-chair-a1b2c3d4").is_ok());
        assert!(OfferSlug::parse("abc").is_ok());
        assert!(OfferSlug::parse("123").is_ok());
        assert!(OfferSlug::parse("a-b-c").is_ok());

        // Too short (< 3)
        assert!(matches!(
            OfferSlug::parse("ab"),
            Err(DomainError::InvalidSlug(_))
        ));
        // Leading hyphen
        assert!(matches!(
            OfferSlug::parse("-abc"),
            Err(DomainError::InvalidSlug(_))
        ));
        // Trailing hyphen
        assert!(matches!(
            OfferSlug::parse("abc-"),
            Err(DomainError::InvalidSlug(_))
        ));
        // Consecutive hyphens
        assert!(matches!(
            OfferSlug::parse("a--b"),
            Err(DomainError::InvalidSlug(_))
        ));
        // Uppercase or special characters
        assert!(matches!(
            OfferSlug::parse("Vintage-chair"),
            Err(DomainError::InvalidSlug(_))
        ));
        assert!(matches!(
            OfferSlug::parse("vintage_chair"),
            Err(DomainError::InvalidSlug(_))
        ));
        assert!(matches!(
            OfferSlug::parse("vintage chair"),
            Err(DomainError::InvalidSlug(_))
        ));

        // Reserved slugs
        assert!(matches!(
            OfferSlug::parse("manage"),
            Err(DomainError::InvalidSlug(_))
        ));
        assert!(matches!(
            OfferSlug::parse("new"),
            Err(DomainError::InvalidSlug(_))
        ));
    }

    #[test]
    fn offer_slug_from_title_and_suffix() {
        let slug =
            OfferSlug::from_title_and_suffix("Vintage Leather Jacket!", "a1b2c3d4ef").unwrap();
        assert_eq!(slug.as_str(), "vintage-leather-jacket-a1b2c3d4");

        // Non-ascii fallback
        let fallback = OfferSlug::from_title_and_suffix("🎉✨🚀", "12345678").unwrap();
        assert_eq!(fallback.as_str(), "offer-12345678");

        // Truncation when title is very long
        let long_title = "a".repeat(120);
        let long_slug = OfferSlug::from_title_and_suffix(&long_title, "abcdef12").unwrap();
        assert!(long_slug.as_str().len() <= 90);
        assert!(long_slug.as_str().ends_with("-abcdef12"));
    }

    #[test]
    fn offer_cursor_encode_decode_roundtrip() {
        let now = Utc::now();
        let offer_id = OfferId::new();
        let cursor = OfferCursor::new(now, offer_id);

        let encoded = cursor.encode();
        let decoded = OfferCursor::decode(&encoded).expect("cursor should decode");

        assert_eq!(decoded.id, offer_id);
        assert_eq!(
            decoded.created_at.timestamp_micros(),
            now.timestamp_micros()
        );

        // Invalid cursor string
        assert!(matches!(
            OfferCursor::decode("not-hex"),
            Err(DomainError::InvalidCursor)
        ));
        assert!(matches!(
            OfferCursor::decode("1234"),
            Err(DomainError::InvalidCursor)
        ));
    }
}
