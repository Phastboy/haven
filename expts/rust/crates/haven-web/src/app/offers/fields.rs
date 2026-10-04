use haven_domain::offer::{CurrencyCode, Price};
use topcoat::router::error::bad_request;
use topcoat::Error as TopcoatError;

/// Parses an optional string into a `Price`.
/// Rejects commas or decimals. Returns a validation error for invalid formats.
fn parse_price_str(raw: &str) -> Result<Price, TopcoatError> {
    let s = raw.trim();
    if s.contains(',') || s.contains('.') {
        return Err(bad_request("Price must be a whole number without commas or decimals").into());
    }
    let value: i32 = s.parse().map_err(|_| bad_request("Invalid price format"))?;
    Price::new(value).map_err(|_| bad_request("Price must be non-negative").into())
}

/// Parses an optional string into a `CurrencyCode`.
fn parse_currency_str(raw: &str) -> Result<CurrencyCode, TopcoatError> {
    CurrencyCode::parse(raw).map_err(|_| bad_request("Invalid currency code").into())
}

/// Parses form fields for an offer creation, substituting defaults if omitted.
pub fn parse_for_create(
    raw_price: Option<&str>,
    raw_currency: Option<&str>,
) -> Result<(Price, CurrencyCode), TopcoatError> {
    let p = match raw_price.map(str::trim) {
        None | Some("") => Price::ZERO,
        Some(s) => parse_price_str(s)?,
    };

    let c = match raw_currency.map(str::trim) {
        None | Some("") => CurrencyCode::default_code(),
        Some(s) => parse_currency_str(s)?,
    };

    Ok((p, c))
}

/// Parses form fields for an offer update. Missing or empty fields mean "leave unchanged" (None).
pub fn parse_for_patch(
    raw_price: Option<&str>,
    raw_currency: Option<&str>,
) -> Result<(Option<Price>, Option<CurrencyCode>), TopcoatError> {
    let p = match raw_price.map(str::trim) {
        None | Some("") => None,
        Some(s) => Some(parse_price_str(s)?),
    };

    let c = match raw_currency.map(str::trim) {
        None | Some("") => None,
        Some(s) => Some(parse_currency_str(s)?),
    };

    Ok((p, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_handles_defaults() {
        let (p, c) = parse_for_create(None, None).unwrap();
        assert_eq!(p.as_i32(), 0);
        assert_eq!(c.as_str(), "NGN");

        let (p, c) = parse_for_create(Some("".into()), Some("  ".into())).unwrap();
        assert_eq!(p.as_i32(), 0);
        assert_eq!(c.as_str(), "NGN");
    }

    #[test]
    fn create_rejects_invalid() {
        assert!(parse_for_create(Some("50,000".into()), None).is_err());
        assert!(parse_for_create(Some("50.0".into()), None).is_err());
        assert!(parse_for_create(Some("abc".into()), None).is_err());
        assert!(parse_for_create(Some("-5".into()), None).is_err());
        assert!(parse_for_create(None, Some("usd1".into())).is_err());
    }

    #[test]
    fn patch_handles_none() {
        let (p, c) = parse_for_patch(None, None).unwrap();
        assert!(p.is_none());
        assert!(c.is_none());

        let (p, c) = parse_for_patch(Some("".into()), Some("  ".into())).unwrap();
        assert!(p.is_none());
        assert!(c.is_none());
    }

    #[test]
    fn patch_parses_values() {
        let (p, c) = parse_for_patch(Some("100".into()), Some("usd".into())).unwrap();
        assert_eq!(p.unwrap().as_i32(), 100);
        assert_eq!(c.unwrap().as_str(), "USD");
    }
}
