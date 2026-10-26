//! Price fields and actions shared by the mobile filter sheet and desktop row.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Values retained in the price controls and the price-only clear destination.
pub struct FeedFilterSheetData {
    /// Currently selected currency code.
    pub currency: String,
    /// Current minimum price in minor units.
    pub min_price: Option<i32>,
    /// Current maximum price in minor units.
    pub max_price: Option<i32>,
    /// Encoded route retaining only the search query.
    pub clear_price_href: String,
}

/// Renders price fields once inside the form's popover dialog.
#[component]
pub async fn feed_filter_sheet(data: FeedFilterSheetData) -> TopcoatResult<impl View> {
    let min_price = data
        .min_price
        .map(|value| value.to_string())
        .unwrap_or_default();
    let max_price = data
        .max_price
        .map(|value| value.to_string())
        .unwrap_or_default();
    Ok(view! {
        <dialog id="feed-filter-sheet" popover="auto" class="filter-sheet">
            <div class="filter-sheet-shell">
                <div class="sheet-grabber" aria-hidden="true"><span></span></div>
                <div class="sheet-header">
                    <h2>"Filters"</h2>
                    <button type="button" class="sheet-close" popovertarget="feed-filter-sheet" popovertargetaction="hide" aria-label="Close filters" autofocus="autofocus">
                        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 6 12 12M18 6 6 18"/></svg>
                    </button>
                </div>
                <div class="sheet-body">
                    <p class="sheet-section-label">"Price"</p>
                    <div class="filters-price-fields">
                        <div class="filter-field filter-field-currency">
                            <label for="filter-currency" class="field-label">"Currency"</label>
                            <select id="filter-currency" name="currency" class="field-input field-select">
                                <option value="" selected=(data.currency.is_empty().then_some("selected"))>"Any"</option>
                                for &curr in haven_domain::offer::SUPPORTED_CURRENCIES {
                                    <option value=(curr) selected=((data.currency == curr).then_some("selected"))>(curr)</option>
                                }
                            </select>
                        </div>
                        <div class="filter-field filter-field-price">
                            <label for="filter-min-price" class="field-label">"Min price"</label>
                            <input id="filter-min-price" class="field-input" type="number" name="min_price" value=(min_price) min="0" placeholder="0" />
                        </div>
                        <div class="filter-field filter-field-price">
                            <label for="filter-max-price" class="field-label">"Max price (0 for Free)"</label>
                            <input id="filter-max-price" class="field-input" type="number" name="max_price" value=(max_price) min="0" placeholder="Max" />
                        </div>
                    </div>
                </div>
                <div class="sheet-footer">
                    <a href=(data.clear_price_href) class="sheet-clear-price">"Clear"</a>
                    <button type="submit" class="button button-primary filter-submit-btn">
                        <span class="sheet-submit-mobile">"Show results"</span>
                        <span class="sheet-submit-desktop">"Filter"</span>
                    </button>
                </div>
            </div>
        </dialog>
    })
}
