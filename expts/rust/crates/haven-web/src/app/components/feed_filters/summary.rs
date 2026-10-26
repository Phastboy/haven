//! Active-filter summary rendered between the controls and offer grid.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Renders the validated filter description and the full-clear link.
#[component]
pub async fn filter_summary(#[into] summary: String) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="filter-summary">
            <p>(summary)</p>
            <a href="/offers" class="filter-clear-link">"Clear filters"</a>
        </div>
    })
}
