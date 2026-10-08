use haven_domain::offer::{CurrencyCode, Price};
use topcoat::Error as TopcoatError;
use topcoat::router::error::bad_request;

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
    CurrencyCode::parse(raw).map_err(|e| bad_request(e.to_string()).into())
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

/// Parsed patch fields for updating an offer.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct OfferPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub currency: Option<CurrencyCode>,
}

/// Parses form fields for an offer update. Missing or empty fields mean "leave unchanged" (None).
pub fn parse_for_patch(
    raw_title: Option<&str>,
    raw_description: Option<&str>,
    raw_price: Option<&str>,
    raw_currency: Option<&str>,
) -> Result<OfferPatch, TopcoatError> {
    let title = match raw_title.map(str::trim) {
        None | Some("") => None,
        Some(s) => {
            haven_domain::offer::validate_title(s).map_err(|e| bad_request(e.to_string()))?;
            Some(s.to_string())
        }
    };

    let description = match raw_description.map(str::trim) {
        None | Some("") => None,
        Some(s) => {
            haven_domain::offer::validate_description(s).map_err(|e| bad_request(e.to_string()))?;
            Some(s.to_string())
        }
    };

    let price = match raw_price.map(str::trim) {
        None | Some("") => None,
        Some(s) => Some(parse_price_str(s)?),
    };

    let currency = match raw_currency.map(str::trim) {
        None | Some("") => None,
        Some(s) => Some(parse_currency_str(s)?),
    };

    Ok(OfferPatch {
        title,
        description,
        price,
        currency,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn create_handles_defaults() {
        let (p, c) = parse_for_create(None, None).unwrap();
        assert_eq!(p.as_i32(), 0);
        assert_eq!(c.as_str(), "NGN");

        let (p, c) = parse_for_create(Some(""), Some("  ")).unwrap();
        assert_eq!(p.as_i32(), 0);
        assert_eq!(c.as_str(), "NGN");
    }

    #[test]
    fn create_rejects_invalid() {
        assert!(parse_for_create(Some("50,000"), None).is_err());
        assert!(parse_for_create(Some("50.0"), None).is_err());
        assert!(parse_for_create(Some("abc"), None).is_err());
        assert!(parse_for_create(Some("-5"), None).is_err());
        assert!(parse_for_create(None, Some("usd1")).is_err());
        assert!(parse_for_create(None, Some("ZZZ")).is_err());
    }

    #[test]
    fn patch_handles_none() {
        let patch = parse_for_patch(None, None, None, None).unwrap();
        assert_eq!(patch, OfferPatch::default());

        let patch = parse_for_patch(Some(""), Some("  "), Some(""), Some("  ")).unwrap();
        assert_eq!(patch, OfferPatch::default());
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
    }
}
