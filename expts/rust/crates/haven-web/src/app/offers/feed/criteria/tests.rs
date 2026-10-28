use super::super::FeedQuery;
use super::ParsedCriteria;

#[test]
fn retains_invalid_price_in_links_while_applying_valid_title() {
    let query = FeedQuery {
        after: None,
        q: Some("  wooden desk  ".to_string()),
        currency: Some("NGN".to_string()),
        min_price: Some(-1),
        max_price: Some(50),
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert!(parsed.criteria.title.is_some());
    assert!(parsed.criteria.price.is_none());
    assert_eq!(
        parsed.controls.price_error.as_deref(),
        Some("Price must be non-negative.")
    );
    assert_eq!(
        parsed.controls.summary.as_deref(),
        Some("Search: wooden desk")
    );
    assert_eq!(
        parsed.href(Some("cursor")),
        "/offers?q=wooden+desk&currency=NGN&min_price=-1&max_price=50&after=cursor"
    );
    assert_eq!(parsed.controls.clear_price_href, "/offers?q=wooden+desk");
}

#[test]
fn currency_without_bounds_stays_in_links_without_becoming_a_filter() {
    let query = FeedQuery {
        after: None,
        q: None,
        currency: Some("unknown".to_string()),
        min_price: None,
        max_price: None,
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert!(parsed.criteria.is_empty());
    assert!(parsed.controls.price_error.is_none());
    assert_eq!(parsed.href(None), "/offers?currency=unknown");
    assert_eq!(ParsedCriteria::parse(None).href(None), "/offers");
}

#[test]
fn free_price_keeps_summary_and_intersects_title() {
    let query = FeedQuery {
        after: None,
        q: Some("chair".to_string()),
        currency: Some("NGN".to_string()),
        min_price: None,
        max_price: Some(0),
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert_eq!(parsed.criteria.iter().count(), 2);
    assert_eq!(
        parsed.controls.summary.as_deref(),
        Some("Search: chair · NGN · Max 0")
    );
    assert_eq!(parsed.controls.price_filter_count, 1);
}
