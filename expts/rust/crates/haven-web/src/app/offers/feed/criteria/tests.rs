use super::super::FeedQuery;
use super::ParsedCriteria;

#[test]
fn retains_invalid_price_in_links_while_applying_valid_title() {
    let query = FeedQuery {
        scope: None,
        category: None,
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
        scope: None,
        category: None,
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
        scope: None,
        category: None,
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

#[test]
fn category_survives_pagination_save_return_and_price_clear() {
    let query = FeedQuery {
        scope: None,
        category: Some("food-groceries".to_string()),
        q: Some("rice".to_string()),
        currency: Some("NGN".to_string()),
        min_price: Some(100),
        max_price: None,
        after: None,
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert_eq!(parsed.criteria.iter().count(), 3);
    assert_eq!(
        parsed.href(Some("cursor")),
        "/offers?category=food-groceries&q=rice&currency=NGN&min_price=100&after=cursor"
    );
    assert_eq!(
        parsed.controls.clear_price_href,
        "/offers?category=food-groceries&q=rice"
    );
    assert_eq!(
        parsed.controls.summary.as_deref(),
        Some("Search: rice · NGN · Min 100")
    );
}

#[test]
fn category_chips_cover_the_vocabulary_in_stable_browse_order() {
    let parsed = ParsedCriteria::parse(None);
    let row = parsed.category_row();
    assert_eq!(row.accessible_name, "Offer categories");
    assert_eq!(
        row.chips.len(),
        haven_domain::offer::Category::ALL.len().saturating_add(1)
    );
    assert_eq!(
        row.chips
            .iter()
            .take(3)
            .map(|chip| chip.label.as_str())
            .collect::<Vec<_>>(),
        ["All", "Food", "Stays"]
    );
    assert_eq!(row.chips.iter().filter(|chip| chip.selected).count(), 1);
    assert_eq!(
        row.chips
            .first()
            .map(|chip| (chip.href.as_str(), chip.selected)),
        Some(("/offers", true))
    );
    for category in haven_domain::offer::Category::ALL {
        let expected = format!("/offers?category={}", category.as_str());
        let chip = row.chips.iter().find(|chip| chip.href == expected);
        assert_eq!(
            chip.and_then(|chip| chip.accessible_label.as_deref()),
            Some(category.label())
        );
    }
}

#[test]
fn switching_category_or_all_preserves_other_input_and_drops_the_cursor() {
    let query = FeedQuery {
        scope: None,
        category: Some("food-groceries".to_string()),
        q: Some("rice & beans".to_string()),
        currency: Some("NGN".to_string()),
        min_price: Some(-1),
        max_price: Some(600),
        after: Some("old-page".to_string()),
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    let row = parsed.category_row();
    assert_eq!(
        row.chips.first().map(|chip| chip.href.as_str()),
        Some("/offers?q=rice+%26+beans&currency=NGN&min_price=-1&max_price=600")
    );
    let stays = row.chips.iter().find(|chip| chip.label == "Stays");
    assert_eq!(
        stays.map(|chip| chip.href.as_str()),
        Some(
            "/offers?category=stays-accommodation&q=rice+%26+beans&currency=NGN&min_price=-1&max_price=600"
        )
    );
    assert!(row.chips.iter().all(|chip| !chip.href.contains("after=")));
    assert_eq!(
        row.chips
            .iter()
            .filter(|chip| chip.selected)
            .map(|chip| chip.label.as_str())
            .collect::<Vec<_>>(),
        ["Food"]
    );
    assert_eq!(
        parsed.controls.clear_filters_href,
        "/offers?category=food-groceries"
    );
    assert_eq!(
        parsed.controls.clear_price_href,
        "/offers?category=food-groceries&q=rice+%26+beans"
    );
}

#[test]
fn category_alone_does_not_duplicate_its_chip_in_the_summary_or_badge() {
    let query = FeedQuery {
        scope: None,
        category: Some("food-groceries".to_string()),
        q: None,
        currency: None,
        min_price: None,
        max_price: None,
        after: None,
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert!(parsed.controls.summary.is_none());
    assert_eq!(parsed.controls.price_filter_count, 0);
}

#[test]
fn unknown_category_is_invalid_without_selecting_all() {
    let query = FeedQuery {
        scope: None,
        category: Some("unknown-topic".to_string()),
        q: None,
        currency: None,
        min_price: None,
        max_price: None,
        after: None,
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert!(parsed.controls.category_error.is_some());
    assert!(
        parsed
            .category_row()
            .chips
            .iter()
            .all(|chip| !chip.selected)
    );
    assert_eq!(parsed.category_href(None), "/offers");
}

#[test]
fn scope_switches_preserve_facets_and_reset_cursors() {
    let query = FeedQuery {
        scope: Some("saved".to_string()),
        category: Some("food-groceries".to_string()),
        q: Some("rice".to_string()),
        currency: Some("NGN".to_string()),
        min_price: Some(10),
        max_price: None,
        after: Some("old-cursor".to_string()),
    };
    let parsed = ParsedCriteria::parse(Some(&query));
    assert_eq!(
        parsed.scope_href(super::super::collection::Collection::Inquired),
        "/offers?scope=inquired&category=food-groceries&q=rice&currency=NGN&min_price=10"
    );
    assert_eq!(
        parsed.category_href(None),
        "/offers?scope=saved&q=rice&currency=NGN&min_price=10"
    );
    assert_eq!(
        parsed.controls.clear_filters_href,
        "/offers?scope=saved&category=food-groceries"
    );
}
