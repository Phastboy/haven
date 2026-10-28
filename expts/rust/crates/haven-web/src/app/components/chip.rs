//! Reusable navigation link with selected and outlined appearances.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Display content and destination for a chip-shaped navigation link.
#[derive(Debug, Clone)]
pub struct ChipData {
    /// Visible, non-wrapping label.
    pub label: String,
    /// Navigation destination supplied by the caller.
    pub href: String,
    /// Whether this destination represents the current selection.
    pub selected: bool,
    /// Optional expanded name for assistive technology.
    pub accessible_label: Option<String>,
}

/// Renders an ordinary anchor; navigation and selection are owned by its caller.
#[component]
pub async fn chip(props: ChipData) -> TopcoatResult<impl View> {
    Ok(view! {
        <a class="chip" href=(props.href)
            aria-current=(props.selected.then_some("true"))
            aria-label=(props.accessible_label)>
            (props.label)
        </a>
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "rendering assertions")]
mod tests;
