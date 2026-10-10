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
    let patch = parse_for_patch(RawOfferPatch::default()).unwrap();
    assert_eq!(patch, OfferPatch::default());
}

#[test]
fn patch_rejects_blank_required_fields() {
    assert!(
        parse_for_patch(RawOfferPatch {
            title: Some(""),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            title: Some("  "),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            description: Some(""),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            description: Some("  "),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            price: Some(""),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            price: Some("  "),
            ..Default::default()
        })
        .is_err()
    );
}

#[test]
fn patch_parses_values() {
    let patch = parse_for_patch(RawOfferPatch {
        title: Some("Valid Offer"),
        description: Some("Detailed description here"),
        price: Some("100"),
        currency: Some("usd"),
    })
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
    assert!(
        parse_for_patch(RawOfferPatch {
            title: Some("ab"),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            description: Some("short"),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            price: Some("abc"),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            currency: Some("ZZZ"),
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        parse_for_patch(RawOfferPatch {
            price: Some("100"),
            currency: Some(""),
            ..Default::default()
        })
        .is_err()
    );
}
