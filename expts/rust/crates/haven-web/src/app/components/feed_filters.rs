//! Responsive search and price-filter form assembly for the public offers feed.

mod sheet;
mod summary;

use super::chip_row::{ChipRowData, chip_row};
use sheet::{FeedFilterSheetData, feed_filter_sheet};
use summary::filter_summary;
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Validated and display-ready values used to render the offers filter form.
pub struct FeedFiltersData {
    /// Selected collection, retained by form submission.
    pub scope: String,
    /// Invalid collection feedback.
    pub scope_error: Option<String>,
    /// Collection navigation supplied by the feed assembly.
    pub scope_row: Option<ChipRowData>,
    /// Submitted category slug, retained even when unknown.
    pub category: String,
    /// Category validation error shown above the topic links.
    pub category_error: Option<String>,
    /// Clears title and price while retaining the selected topic.
    pub clear_filters_href: String,
    /// Current search field value, retained even when invalid.
    pub title_query: String,
    /// Current currency field value, retained even when invalid.
    pub currency: String,
    /// Current minimum price field value.
    pub min_price: Option<i32>,
    /// Current maximum price field value.
    pub max_price: Option<i32>,
    /// Search validation error, if any.
    pub title_error: Option<String>,
    /// Price validation error, if any.
    pub price_error: Option<String>,
    /// Number of active price filters to show in the trigger badge.
    pub price_filter_count: u32,
    /// Summary of accepted filters, if any.
    pub summary: Option<String>,
    /// Encoded URL that clears price fields while retaining the search query.
    pub clear_price_href: String,
}

/// Renders the one-form responsive filters control and its native popover sheet.
#[component]
pub async fn feed_filters(data: FeedFiltersData) -> TopcoatResult<impl View> {
    let sheet_data = FeedFilterSheetData {
        currency: data.currency.clone(),
        min_price: data.min_price,
        max_price: data.max_price,
        clear_price_href: data.clear_price_href,
    };

    Ok(view! {
        <form action="/offers" method="get" class="feed-filters" role="search" aria-label="Filter offers">
            <input type="hidden" name="scope" value=(data.scope) />
            if let Some(row) = data.scope_row { chip_row(props: row) }
            if let Some(error) = data.scope_error { <p class="feed-filter-error" role="alert">(error)</p> }
            <input type="hidden" name="category" value=(data.category) />
            <div class="filter-header-row">
                <div class="filter-field filter-field-title">
                    <label for="filter-q" class="field-label">"Find by title"</label>
                    <input
                        id="filter-q"
                        class="field-input"
                        type="search"
                        name="q"
                        value=(data.title_query)
                        placeholder="Search offers by name..."
                        maxlength="100"
                        autocomplete="off"
                        enterkeyhint="search"
                    />
                </div>
                <button
                    type="button"
                    class="filter-trigger"
                    popovertarget="feed-filter-sheet"
                    aria-label=(if data.price_filter_count > 0 { format!("Filters, {} price filter active", data.price_filter_count) } else { "Filters".to_string() })
                >
                    <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 7h16M7 12h10m-7 5h4"/><circle cx="8" cy="7" r="1" fill="currentColor"/><circle cx="15" cy="12" r="1" fill="currentColor"/></svg>
                    <span>"Filters"</span>
                    if data.price_filter_count > 0 {
                        <span class="filter-count-badge" aria-label=(format!("{} active price filters", data.price_filter_count))>(data.price_filter_count)</span>
                    }
                </button>
            </div>

            if let Some(error) = data.category_error {
                <p class="feed-filter-error" role="alert">(error)</p>
            }
            if let Some(error) = data.title_error {
                <p class="feed-filter-error" role="alert">(error)</p>
            }
            if let Some(error) = data.price_error {
                <p class="feed-filter-error" role="alert">(error)</p>
            }
            if let Some(summary) = data.summary {
                filter_summary(summary: summary, clear_href: data.clear_filters_href)
            }

            feed_filter_sheet(data: sheet_data)
        </form>
    })
}

/// Encodes the query retained by the sheet's price-only clear link.
#[must_use]
pub fn clear_price_href(query: &str) -> String {
    if query.is_empty() {
        return "/offers".to_string();
    }
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("q", query);
    format!("/offers?{}", serializer.finish())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "component assertions")]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    fn data(count: u32, summary: Option<&str>) -> FeedFiltersData {
        FeedFiltersData {
            scope: "public".to_string(),
            scope_error: None,
            scope_row: None,
            category: String::new(),
            category_error: None,
            clear_filters_href: "/offers".to_string(),
            title_query: "mug & bowl".to_string(),
            currency: "USD".to_string(),
            min_price: None,
            max_price: Some(30),
            title_error: None,
            price_error: None,
            price_filter_count: count,
            summary: summary.map(str::to_string),
            clear_price_href: clear_price_href("mug & bowl"),
        }
    }

    #[tokio::test]
    async fn renders_one_form_one_copy_of_each_field_and_price_count() {
        let cx = Cx::default();
        let __cx = &cx;
        let rendered = view! { feed_filters(data: data(1, Some("Search: mug · USD · Max 30"))) };
        let html = rendered.single().await.unwrap().render(__cx);
        assert!(html.contains("method=\"get\""));
        assert!(html.contains("id=\"feed-filter-sheet\" popover=\"auto\""));
        assert!(html.contains("popovertarget=\"feed-filter-sheet\""));
        for field in ["category", "q", "currency", "min_price", "max_price"] {
            assert_eq!(html.matches(&format!("name=\"{field}\"")).count(), 1);
        }
        assert!(html.contains("class=\"filter-count-badge\""));
        assert!(html.contains("Search: mug · USD · Max 30"));
        assert!(html.contains("/offers?q=mug+%26+bowl"));
    }

    #[tokio::test]
    async fn omits_count_badge_without_active_price_filter() {
        let cx = Cx::default();
        let __cx = &cx;
        let rendered = view! { feed_filters(data: data(0, None)) };
        let html = rendered.single().await.unwrap().render(__cx);
        assert!(!html.contains("filter-count-badge"));
        assert!(!html.contains("filter-summary"));
    }
    #[tokio::test]
    async fn submitting_controls_retains_the_category_without_a_second_selector() {
        let cx = Cx::default();
        let __cx = &cx;
        let mut fields = data(1, Some("Search: rice · NGN · Max 600"));
        fields.category = "food-groceries".to_string();
        fields.clear_filters_href = "/offers?category=food-groceries".to_string();
        let rendered = view! { feed_filters(data: fields) };
        let html = rendered.single().await.unwrap().render(__cx);
        assert!(html.contains("type=\"hidden\" name=\"category\" value=\"food-groceries\""));
        assert_eq!(html.matches("name=\"category\"").count(), 1);
        assert!(!html.contains("id=\"filter-category\""));
        assert!(
            html.contains("href=\"/offers?category=food-groceries\" class=\"filter-clear-link\"")
        );
    }
}
