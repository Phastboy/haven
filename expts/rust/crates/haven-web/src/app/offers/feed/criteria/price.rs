//! Price validation, summary and retained query-string contribution.

use crate::app::components::feed_filters::FeedFiltersData;
use haven_domain::offer::{CurrencyCode, Price, PriceFilter};

pub(super) fn parse(
    currency: &str,
    min: Option<i32>,
    max: Option<i32>,
) -> (Option<PriceFilter>, Option<String>) {
    if min.is_none() && max.is_none() {
        return (None, None);
    }
    if currency.is_empty() {
        return (
            None,
            Some("Currency is required when filtering by price.".to_string()),
        );
    }
    let Ok(currency) = CurrencyCode::parse(currency) else {
        return (None, Some("Invalid currency code.".to_string()));
    };
    let (Ok(min), Ok(max)) = (
        min.map(Price::new).transpose(),
        max.map(Price::new).transpose(),
    ) else {
        return (None, Some("Price must be non-negative.".to_string()));
    };
    match PriceFilter::new(currency, min, max) {
        Ok(price) => (Some(price), None),
        Err(error) => (None, Some(error.to_string())),
    }
}

pub(super) fn append_url(
    serializer: &mut url::form_urlencoded::Serializer<'_, String>,
    data: &FeedFiltersData,
) {
    if !data.currency.is_empty() {
        serializer.append_pair("currency", &data.currency);
    }
    if let Some(min) = data.min_price {
        serializer.append_pair("min_price", &min.to_string());
    }
    if let Some(max) = data.max_price {
        serializer.append_pair("max_price", &max.to_string());
    }
}

pub(super) fn summarize(price: &PriceFilter, parts: &mut Vec<String>) {
    parts.push(price.currency.as_str().to_string());
    if let Some(min) = price.min {
        parts.push(format!("Min {}", min.as_i32()));
    }
    if let Some(max) = price.max {
        parts.push(format!("Max {}", max.as_i32()));
    }
}
