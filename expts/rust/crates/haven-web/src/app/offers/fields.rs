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

/// Parses form fields for an offer creation.
///
/// Price must be explicitly stated even if it is zero.
/// When price is zero, currency is optional and defaults to `CurrencyCode::default_code()`.
/// When price is greater than zero, currency is explicitly required.
pub fn parse_for_create(
    raw_price: Option<&str>,
    raw_currency: Option<&str>,
) -> Result<(Price, CurrencyCode), TopcoatError> {
    let price_str = match raw_price.map(str::trim) {
        None | Some("") => return Err(bad_request("Price is required").into()),
        Some(s) => s,
    };
    let price = parse_price_str(price_str)?;

    let currency = match raw_currency.map(str::trim) {
        None | Some("") => {
            if price.as_i32() > 0 {
                return Err(
                    bad_request("Currency is required when price is greater than zero").into(),
                );
            }
            CurrencyCode::default_code()
        }
        Some(s) => parse_currency_str(s)?,
    };

    Ok((price, currency))
}

/// Parsed patch fields for updating an offer.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct OfferPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub currency: Option<CurrencyCode>,
}

/// Raw unparsed form fields submitted for updating an offer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RawOfferPatch<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price: Option<&'a str>,
    pub currency: Option<&'a str>,
}

/// Parses form fields for an offer update. Missing fields (None) mean "leave unchanged",
/// while empty strings for required fields return validation errors.
/// Currency is required when price is explicitly set greater than zero.
pub fn parse_for_patch(raw: RawOfferPatch<'_>) -> Result<OfferPatch, TopcoatError> {
    let title = match raw.title.map(str::trim) {
        None => None,
        Some("") => return Err(bad_request("Title must not be blank").into()),
        Some(s) => {
            haven_domain::offer::validate_title(s).map_err(|e| bad_request(e.to_string()))?;
            Some(s.to_string())
        }
    };

    let description = match raw.description.map(str::trim) {
        None => None,
        Some("") => return Err(bad_request("Description must not be blank").into()),
        Some(s) => {
            haven_domain::offer::validate_description(s).map_err(|e| bad_request(e.to_string()))?;
            Some(s.to_string())
        }
    };

    let price = match raw.price.map(str::trim) {
        None => None,
        Some("") => return Err(bad_request("Price is required").into()),
        Some(s) => Some(parse_price_str(s)?),
    };

    let currency = match raw.currency.map(str::trim) {
        None | Some("") => None,
        Some(s) => Some(parse_currency_str(s)?),
    };

    if price.is_some_and(|p| p.as_i32() > 0) && matches!(raw.currency.map(str::trim), Some("")) {
        return Err(bad_request("Currency is required when price is greater than zero").into());
    }

    Ok(OfferPatch {
        title,
        description,
        price,
        currency,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests;
