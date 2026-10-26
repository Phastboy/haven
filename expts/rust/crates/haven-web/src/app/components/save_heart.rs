//! Save or unsave control used on public discovery cards.

use haven_domain::offer::OfferSlug;
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Inputs needed to render an offer save control.
pub struct SaveHeartData {
    /// Offer slug used by the save endpoint.
    pub slug: OfferSlug,
    /// Whether the current person has saved this offer.
    pub is_saved: bool,
    /// Feed URL to return to after the save action.
    pub return_to: Option<String>,
    /// Whether to show a visible Save/Saved label beside the heart.
    pub show_label: bool,
    /// Whether a signed-in person is viewing the card.
    pub is_authenticated: bool,
}

/// Renders an accessible save control without client-side code.
#[component]
pub async fn save_heart(props: SaveHeartData) -> TopcoatResult<impl View> {
    let label = if props.is_saved {
        "Remove saved offer"
    } else {
        "Save offer"
    };
    let action = if props.is_saved { "unsave" } else { "save" };
    let return_to = props.return_to;
    let save_label = if props.is_saved { "Saved" } else { "Save" };
    Ok(view! {
        if props.is_authenticated {
            <form class="save-heart-form" action=(format!("/offers/{}/save", props.slug.as_str())) method="post">
                <input type="hidden" name="action" value=(action) />
                if let Some(return_to) = return_to {
                    <input type="hidden" name="return_to" value=(return_to) />
                }
                <button class="save-heart" type="submit" aria-label=(label) title=(label)>
                    <svg viewBox="0 0 24 24" aria-hidden="true" fill=(if props.is_saved { "currentColor" } else { "none" }) stroke="currentColor" stroke-width="2"><path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8l1.1 1.1L12 21l7.8-7.5 1.1-1.1a5.5 5.5 0 0 0-.1-7.8Z" /></svg>
                    if props.show_label { <span>(save_label)</span> }
                </button>
            </form>
        } else {
            <a class="save-heart" href="/auth/sign-in" aria-label="Sign in to save offer" title="Sign in to save offer"><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8l1.1 1.1L12 21l7.8-7.5 1.1-1.1a5.5 5.5 0 0 0-.1-7.8Z" /></svg>{if props.show_label { <span>"Save"</span> }}</a>
        }
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "component test fixtures and assertions")]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    async fn save_heart_renders_post_for_signed_in_person_and_link_for_guest() {
        let cx = Cx::default();
        let __cx = &cx;
        let slug = OfferSlug::parse("wood-chair").expect("valid test slug");
        let signed_in = view! { save_heart(props: SaveHeartData {
            slug: slug.clone(), is_saved: false, return_to: Some("/offers?q=chair".into()), show_label: false, is_authenticated: true,
        }) };
        let html = signed_in
            .single()
            .await
            .expect("single component")
            .render(__cx);
        assert!(html.contains("method=\"post\""));
        assert!(html.contains("name=\"return_to\" value=\"/offers?q=chair\""));
        assert!(html.contains("aria-label=\"Save offer\""));

        let guest_slug = OfferSlug::parse("wood-chair").expect("valid test slug");
        let guest = view! { save_heart(props: SaveHeartData {
            slug: guest_slug, is_saved: false, return_to: None, show_label: false, is_authenticated: false,
        }) };
        let html = guest.single().await.expect("single component").render(__cx);
        assert!(html.contains("href=\"/auth/sign-in\""));
        assert!(html.contains("Sign in to save offer"));
    }
}
