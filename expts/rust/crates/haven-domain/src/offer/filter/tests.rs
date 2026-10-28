use super::*;
use crate::error::FilterValidationError;
use crate::offer::currency::CurrencyCode;
use crate::offer::price::Price;

#[test]
fn test_search_query_parse_success() {
    let q = SearchQuery::parse("  wooden desk  ").unwrap();
    assert_eq!(q.as_str(), "wooden desk");
    assert_eq!(q.to_string(), "wooden desk");
}

#[test]
fn test_search_query_parse_empty() {
    assert_eq!(
        SearchQuery::parse("   ").unwrap_err(),
        FilterValidationError::BlankSearchQuery
    );
    assert_eq!(
        SearchQuery::parse("").unwrap_err(),
        FilterValidationError::BlankSearchQuery
    );
}

#[test]
fn test_search_query_parse_too_long() {
    let long_str = "a".repeat(101);
    assert_eq!(
        SearchQuery::parse(long_str).unwrap_err(),
        FilterValidationError::SearchQueryTooLong
    );
}

#[test]
fn test_price_filter_validation() {
    let min = Price::new(100).unwrap();
    let max = Price::new(500).unwrap();

    // Valid range
    let pf = PriceFilter::new(CurrencyCode::Ngn, Some(min), Some(max)).unwrap();
    assert_eq!(pf.currency, CurrencyCode::Ngn);
    assert_eq!(pf.min, Some(min));
    assert_eq!(pf.max, Some(max));
    assert!(!pf.is_free_only());

    // Inverted range
    let err = PriceFilter::new(CurrencyCode::Ngn, Some(max), Some(min)).unwrap_err();
    assert_eq!(err, FilterValidationError::InvalidPriceRange);

    // Empty filter (both None)
    let err_empty = PriceFilter::new(CurrencyCode::Ngn, None, None).unwrap_err();
    assert_eq!(err_empty, FilterValidationError::EmptyPriceFilter);

    // Free filter
    let free_pf = PriceFilter::free(CurrencyCode::Usd);
    assert_eq!(free_pf.currency, CurrencyCode::Usd);
    assert_eq!(free_pf.min, Some(Price::ZERO));
    assert_eq!(free_pf.max, Some(Price::ZERO));
    assert!(free_pf.is_free_only());
}

#[test]
fn test_public_criteria_empty() {
    let filter = PublicCriteria::default();
    assert!(filter.is_empty());

    let with_title = PublicCriteria {
        title: Some(SearchQuery::parse("test").unwrap()),
        price: None,
    };
    assert!(!with_title.is_empty());
}

#[test]
fn public_criteria_composes_title_and_price_in_order() {
    let criteria = PublicCriteria {
        title: Some(SearchQuery::parse("  wooden desk ").unwrap()),
        price: Some(PriceFilter::free(CurrencyCode::Ngn)),
    };
    let mut facets = criteria.iter();
    assert!(
        matches!(facets.next(), Some(PublicCriterion::Title(title)) if title.as_str() == "wooden desk")
    );
    assert!(matches!(facets.next(), Some(PublicCriterion::Price(price)) if price.is_free_only()));
    assert!(facets.next().is_none());
}

#[test]
fn public_feed_query_keeps_scope_separate_from_facets() {
    let criteria = PublicCriteria {
        title: None,
        price: Some(PriceFilter::free(CurrencyCode::Usd)),
    };
    let query = crate::offer::PublicFeedQuery::new(criteria.clone(), None, 20);
    assert_eq!(query.scope(), Scope::Public);
    assert_eq!(query.criteria, criteria);
    assert_eq!(
        crate::offer::PublicFeedQuery::default().scope(),
        Scope::Public
    );
}
