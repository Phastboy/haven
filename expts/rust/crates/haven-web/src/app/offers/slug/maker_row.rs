//! Maker identity row on the public offer page.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Maker row presentation data.
pub struct MakerRowData {
    /// Maker's display name.
    pub display_name: String,
    /// Maker profile URL.
    pub profile_href: String,
    /// Joined date in the shared display format.
    pub joined_date: String,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// Optional 1x/2x avatar srcset.
    pub avatar_srcset: Option<String>,
    /// Fallback initial when there is no avatar.
    pub initial: String,
}

/// Renders the linked maker attribution.
#[component]
pub async fn maker_row(data: MakerRowData) -> TopcoatResult<impl View> {
    Ok(view! {
        <a class="offer-maker" href=(data.profile_href)>
            if let Some(avatar_url) = data.avatar_url {
                <img class="offer-maker-avatar" src=(avatar_url) srcset=(data.avatar_srcset.unwrap_or_default()) alt="" width="48" height="48" />
            } else {
                <span class="offer-maker-avatar offer-maker-initial" aria-hidden="true">(data.initial)</span>
            }
            <span class="offer-maker-copy">
                <span class="offer-maker-name"><span class="text-muted">"Offered by "</span><strong>(data.display_name)</strong></span>
                <span class="offer-maker-joined">(data.joined_date)</span>
            </span>
        </a>
    })
}
