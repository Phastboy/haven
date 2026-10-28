//! Active-filter summary rendered between the controls and offer grid.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Renders the validated filter description and the clear destination supplied by its caller.
#[component]
pub async fn filter_summary(
    #[into] summary: String,
    #[into] clear_href: String,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="filter-summary">
            <p>(summary)</p>
            <a href=(clear_href) class="filter-clear-link">"Clear filters"</a>
        </div>
    })
}
