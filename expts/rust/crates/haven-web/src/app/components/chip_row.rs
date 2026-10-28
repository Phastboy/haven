//! Responsive navigation row with CSS-only overflow and wrapping.

use super::chip::{ChipData, chip};
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Ordered links and an accessible name for their navigation region.
pub struct ChipRowData {
    /// Links in the order chosen by the caller.
    pub chips: Vec<ChipData>,
    /// Describes the purpose of this navigation region.
    pub accessible_name: String,
}

/// Scrolls on narrow screens and wraps at the desktop feed breakpoint.
///
/// Empty rows render nothing, allowing page assembly to reserve a row slot.
#[component]
pub async fn chip_row(props: ChipRowData) -> TopcoatResult<impl View> {
    Ok(view! {
        if !props.chips.is_empty() {
            <nav class="chip-row" aria-label=(props.accessible_name)>
                <ul class="chip-row-list" role="list">
                    for props in props.chips {
                        <li>chip(props: props)</li>
                    }
                </ul>
            </nav>
        }
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "rendering assertions")]
mod tests;
