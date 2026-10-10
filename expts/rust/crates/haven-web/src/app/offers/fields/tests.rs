use super::*;

#[test]
fn create_handles_explicit_zero_price() {
    let (p, c) = parse_for_create(Some("0"), None).unwrap();
    assert_eq!(p.as_i32(), 0);
    assert_eq!(c.as_str(), "NGN");

    let (p, c) = parse_for_create(Some("0"), Some("")).unwrap();
    assert_eq!(p.as_i32(), 0);
    assert_eq!(c.as_str(), "NGN");

    let (p, c) = parse_for_create(Some("0"), Some("USD")).unwrap();
    assert_eq!(p.as_i32(), 0);
    assert_eq!(c.as_str(), "USD");
}

#[test]
fn create_requires_explicit_price() {
    assert!(parse_for_create(None, None).is_err());
    assert!(parse_for_create(Some(""), None).is_err());
    assert!(parse_for_create(Some("  "), None).is_err());
    assert!(parse_for_create(None, Some("USD")).is_err());
}

#[test]
fn create_requires_currency_when_price_positive() {
    assert!(parse_for_create(Some("100"), None).is_err());
    assert!(parse_for_create(Some("100"), Some("")).is_err());
    assert!(parse_for_create(Some("100"), Some("  ")).is_err());

    let (p, c) = parse_for_create(Some("100"), Some("USD")).unwrap();
    assert_eq!(p.as_i32(), 100);
    assert_eq!(c.as_str(), "USD");
}

#[test]
fn create_rejects_invalid() {
    assert!(parse_for_create(Some("50,000"), Some("USD")).is_err());
    assert!(parse_for_create(Some("50.0"), Some("USD")).is_err());
    assert!(parse_for_create(Some("abc"), Some("USD")).is_err());
    assert!(parse_for_create(Some("-5"), Some("USD")).is_err());
    assert!(parse_for_create(Some("0"), Some("usd1")).is_err());
    assert!(parse_for_create(Some("0"), Some("ZZZ")).is_err());
    assert!(parse_for_create(Some("100"), Some("ZZZ")).is_err());
}

#[test]
fn patch_handles_none() {
    let patch = parse_for_patch(None, None, None, None).unwrap();
    assert_eq!(patch, OfferPatch::default());
}

#[test]
fn patch_rejects_blank_required_fields() {
    assert!(parse_for_patch(Some(""), None, None, None).is_err());
    assert!(parse_for_patch(Some("  "), None, None, None).is_err());
    assert!(parse_for_patch(None, Some(""), None, None).is_err());
    assert!(parse_for_patch(None, Some("  "), None, None).is_err());
    assert!(parse_for_patch(None, None, Some(""), None).is_err());
    assert!(parse_for_patch(None, None, Some("  "), None).is_err());
}

#[test]
fn patch_parses_values() {
    let patch = parse_for_patch(
        Some("Valid Offer"),
        Some("Detailed description here"),
        Some("100"),
        Some("usd"),
    )
    .unwrap();
    assert_eq!(patch.title.as_deref(), Some("Valid Offer"));
    assert_eq!(
        patch.description.as_deref(),
        Some("Detailed description here")
    );
    assert_eq!(patch.price.unwrap().as_i32(), 100);
    assert_eq!(patch.currency.unwrap().as_str(), "USD");
}

#[test]
fn patch_rejects_invalid_values() {
    assert!(parse_for_patch(Some("ab"), None, None, None).is_err());
    assert!(parse_for_patch(None, Some("short"), None, None).is_err());
    assert!(parse_for_patch(None, None, Some("abc"), None).is_err());
    assert!(parse_for_patch(None, None, None, Some("ZZZ")).is_err());
    assert!(parse_for_patch(None, None, Some("100"), Some("")).is_err());
}
