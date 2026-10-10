use super::*;
use crate::error::{CursorError, OfferValidationError};

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
    assert!(matches!(
        validate_title(""),
        Err(OfferValidationError::BlankTitle)
    ));
    assert!(matches!(
        validate_title("   "),
        Err(OfferValidationError::BlankTitle)
    ));
    assert!(matches!(
        validate_title("ab"),
        Err(OfferValidationError::TitleTooShort)
    ));
    assert!(validate_title("abc").is_ok());

    let exactly_100_chars = "a".repeat(100);
    assert!(validate_title(&exactly_100_chars).is_ok());

    let over_100_chars = "a".repeat(101);
    assert!(matches!(
        validate_title(&over_100_chars),
        Err(OfferValidationError::TitleTooLong)
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
        Err(OfferValidationError::BlankDescription)
    ));
    assert!(matches!(
        validate_description("   "),
        Err(OfferValidationError::BlankDescription)
    ));
    assert!(matches!(
        validate_description("123456789"),
        Err(OfferValidationError::DescriptionTooShort)
    ));
    assert!(validate_description("1234567890").is_ok());

    let exactly_2000_chars = "x".repeat(2000);
    assert!(validate_description(&exactly_2000_chars).is_ok());

    let over_2000_chars = "x".repeat(2001);
    assert!(matches!(
        validate_description(&over_2000_chars),
        Err(OfferValidationError::DescriptionTooLong)
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
    assert!(matches!(
        co.validate(),
        Err(OfferValidationError::BlankTitle)
    ));

    co.title = "Valid Title".into();
    co.description = "short".into();
    assert!(matches!(
        co.validate(),
        Err(OfferValidationError::DescriptionTooShort)
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
        Err(OfferValidationError::TitleTooShort)
    ));

    let uo_invalid_desc = UpdateOffer {
        title: None,
        description: Some("short".into()),
        price: None,
        currency: None,
    };
    assert!(matches!(
        uo_invalid_desc.validate(),
        Err(OfferValidationError::DescriptionTooShort)
    ));

    let uo_overlong_title = UpdateOffer {
        title: Some("a".repeat(101)),
        description: None,
        price: None,
        currency: None,
    };
    assert!(matches!(
        uo_overlong_title.validate(),
        Err(OfferValidationError::TitleTooLong)
    ));

    let uo_overlong_desc = UpdateOffer {
        title: None,
        description: Some("x".repeat(2001)),
        price: None,
        currency: None,
    };
    assert!(matches!(
        uo_overlong_desc.validate(),
        Err(OfferValidationError::DescriptionTooLong)
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
    assert!(matches!(
        co.validate(),
        Err(OfferValidationError::TitleTooLong)
    ));

    co.title = "Valid Title".into();
    co.description = "x".repeat(2001);
    assert!(matches!(
        co.validate(),
        Err(OfferValidationError::DescriptionTooLong)
    ));
}

#[test]
fn all_supported_currencies_parse_and_roundtrip() {
    for &code in SUPPORTED_CURRENCIES {
        let parsed = CurrencyCode::parse(code).expect("supported currency should parse");
        assert_eq!(parsed.as_str(), code);

        let lower = code.to_lowercase();
        let parsed_lower = CurrencyCode::parse(&lower).expect("lowercase currency should parse");
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
        Err(OfferValidationError::TitleTooLong)
    ));

    // 2000 emoji characters = 8000 bytes, but exactly 2000 characters
    let desc_2000_emojis = "✨".repeat(2000);
    assert_eq!(desc_2000_emojis.chars().count(), 2000);
    assert!(validate_description(&desc_2000_emojis).is_ok());

    // 2001 emoji characters -> Too long
    let desc_2001_emojis = "✨".repeat(2001);
    assert!(matches!(
        validate_description(&desc_2001_emojis),
        Err(OfferValidationError::DescriptionTooLong)
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
        Err(OfferValidationError::CurrencyRequired)
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
        Err(OfferValidationError::InvalidSlug(_))
    ));
    // Leading hyphen
    assert!(matches!(
        OfferSlug::parse("-abc"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    // Trailing hyphen
    assert!(matches!(
        OfferSlug::parse("abc-"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    // Consecutive hyphens
    assert!(matches!(
        OfferSlug::parse("a--b"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    // Uppercase or special characters
    assert!(matches!(
        OfferSlug::parse("Vintage-chair"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    assert!(matches!(
        OfferSlug::parse("vintage_chair"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    assert!(matches!(
        OfferSlug::parse("vintage chair"),
        Err(OfferValidationError::InvalidSlug(_))
    ));

    // Reserved slugs
    assert!(matches!(
        OfferSlug::parse("manage"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
    assert!(matches!(
        OfferSlug::parse("new"),
        Err(OfferValidationError::InvalidSlug(_))
    ));
}

#[test]
fn offer_slug_from_title_and_suffix() {
    let slug = OfferSlug::from_title_and_suffix("Vintage Leather Jacket!", "a1b2c3d4ef").unwrap();
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
    let offer_id = OfferId::generate();
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
        Err(CursorError::InvalidCursor)
    ));
    assert!(matches!(
        OfferCursor::decode("1234"),
        Err(CursorError::InvalidCursor)
    ));
}

#[test]
fn price_serde_validates_on_deserialization() {
    assert!(serde_json::from_str::<Price>("-1").is_err());
    let zero: Price = serde_json::from_str("0").unwrap();
    assert_eq!(zero.as_i32(), 0);
    let positive: Price = serde_json::from_str("500").unwrap();
    assert_eq!(positive.as_i32(), 500);
}

#[test]
fn offer_slug_serde_validates_on_deserialization() {
    assert!(serde_json::from_str::<OfferSlug>(r#""ab""#).is_err());
    assert!(serde_json::from_str::<OfferSlug>(r#""manage""#).is_err());
    assert!(serde_json::from_str::<OfferSlug>(r#""invalid_slug""#).is_err());
    let valid: OfferSlug = serde_json::from_str(r#""valid-slug-123""#).unwrap();
    assert_eq!(valid.as_str(), "valid-slug-123");
}

#[test]
fn currency_code_serde_validates_on_deserialization() {
    assert!(serde_json::from_str::<CurrencyCode>(r#""INVALID""#).is_err());
    let ngn: CurrencyCode = serde_json::from_str(r#""NGN""#).unwrap();
    assert_eq!(ngn, CurrencyCode::Ngn);
}
