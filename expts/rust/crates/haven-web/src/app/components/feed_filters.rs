//! Filter form component for searching by title and narrowing offers by price.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// A search and price-narrowing filter form component for the public offers feed.
#[component]
pub async fn feed_filters(
    #[default] title_query: String,
    #[default] currency: String,
    #[default] min_price: Option<i32>,
    #[default] max_price: Option<i32>,
    #[default] title_error: Option<String>,
    #[default] price_error: Option<String>,
    #[default] active_filters_count: usize,
) -> TopcoatResult<impl View> {
    let min_str = min_price.map(|p| p.to_string()).unwrap_or_default();
    let max_str = max_price.map(|p| p.to_string()).unwrap_or_default();

    Ok(view! {
        <form action="/offers" method="GET" class="feed-filters" role="search" aria-label="Filter offers">
            <div class="filters-layout">
                <div class="filter-field filter-field-title">
                    <label for="filter-q" class="field-label">
                        "Find by title"
                    </label>
                    <input
                        id="filter-q"
                        class="field-input"
                        type="search"
                        name="q"
                        value=(title_query)
                        placeholder="Search offers by name..."
                        maxlength="100"
                        autocomplete="off"
                    />
                    if let Some(err) = title_error {
                        <p class="field-error" role="alert">(err)</p>
                    }
                </div>

                <div class="filters-price-group">
                    <div class="filter-field filter-field-currency">
                        <label for="filter-currency" class="field-label">
                            "Currency"
                        </label>
                        <select id="filter-currency" name="currency" class="field-input field-select">
                            <option value="" selected=(currency.is_empty().then_some("selected"))>
                                "Any"
                            </option>
                            for &curr in haven_domain::offer::SUPPORTED_CURRENCIES {
                                <option value=(curr) selected=((currency == curr).then_some("selected"))>
                                    (curr)
                                </option>
                            }
                        </select>
                    </div>

                    <div class="filter-field filter-field-price">
                        <label for="filter-min-price" class="field-label">
                            "Min price"
                        </label>
                        <input
                            id="filter-min-price"
                            class="field-input"
                            type="number"
                            name="min_price"
                            value=(min_str)
                            min="0"
                            placeholder="0"
                        />
                    </div>

                    <div class="filter-field filter-field-price">
                        <label for="filter-max-price" class="field-label">
                            "Max price (0 for Free)"
                        </label>
                        <input
                            id="filter-max-price"
                            class="field-input"
                            type="number"
                            name="max_price"
                            value=(max_str)
                            min="0"
                            placeholder="Max"
                        />
                    </div>
                </div>
            </div>

            if let Some(err) = price_error {
                <p class="field-error" role="alert">(err)</p>
            }

            <div class="filter-actions">
                <button type="submit" class="button button-primary filter-submit-btn">
                    "Filter"
                </button>
                if active_filters_count > 0 {
                    <a href="/offers" class="filter-clear-link">
                        "Clear filters"
                    </a>
                }
            </div>
        </form>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn feed_filters_renders_inputs_and_values() {
        let cx = Cx::default();
        let __cx = &cx;
        let el = view! {
            feed_filters(
                title_query: "lamp".to_string(),
                currency: "NGN".to_string(),
                min_price: Some(100),
                max_price: Some(500),
                title_error: None,
                price_error: None,
                active_filters_count: 2,
            )
        };

        let html = el.single().await.unwrap().render(__cx);
        assert!(html.contains("value=\"lamp\""));
        assert!(html.contains("value=\"100\""));
        assert!(html.contains("value=\"500\""));
        assert!(html.contains("Clear filters"));
    }
}
