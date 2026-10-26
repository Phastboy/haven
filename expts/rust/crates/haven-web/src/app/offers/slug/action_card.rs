//! Message actions for the public offer page.

use super::viewer::OfferViewer;
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Data required to render the role-specific message card.
pub struct ActionCardData {
    /// Offer slug used in the POST target.
    pub slug: String,
    /// Price shown in the card.
    pub price: String,
    /// Maker's display name used in the helper line.
    pub maker_name: String,
    /// Requester's role.
    pub role: OfferViewer,
}

/// Renders the role-specific message card.
#[component]
pub async fn action_card(data: ActionCardData) -> TopcoatResult<impl View> {
    Ok(view! {
        <section class="offer-action-card" id="message">
            <div class="offer-action-price">(data.price)</div>
            match data.role {
                OfferViewer::Visitor { .. } => {
                    <div class="offer-action-divider"></div>
                    <label for="offer-message">"Message about this offer"</label>
                    <form action=(format!("/offers/{}/message", data.slug)) method="post" class="offer-message-form">
                        <textarea id="offer-message" name="body" rows="4" maxlength="2000" required="required" placeholder="Ask a question or express your interest..."></textarea>
                        <button type="submit" class="offer-primary-button">"Send message"</button>
                    </form>
                    <p class="offer-action-note">"Sent privately to "(data.maker_name)"."</p>
                }
                OfferViewer::Anonymous => {
                    <a class="offer-primary-button" href="/auth/sign-in">"Sign in to message"</a>
                    <p class="offer-action-note">"You need an account to message "(data.maker_name)"."</p>
                }
                OfferViewer::Owner => {
                    <p class="offer-action-note">"This is your offer preview."</p>
                }
            }
        </section>
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
    async fn renders_role_specific_message_actions() {
        let cx = Cx::default();
        let __cx = &cx;
        for (role, expected) in [
            (OfferViewer::Anonymous, "Sign in to message"),
            (OfferViewer::Visitor { saved: false }, "Send message"),
            (OfferViewer::Owner, "This is your offer preview."),
        ] {
            let component = view! { action_card(data: ActionCardData {
                slug: "offer-slug".into(), price: "25 NGN".into(), maker_name: "Maker".into(), role,
            }) };
            let html = component
                .single()
                .await
                .expect("one action card")
                .render(__cx);
            assert!(html.contains(expected));
            assert_eq!(
                html.contains("name=\"body\""),
                matches!(role, OfferViewer::Visitor { .. })
            );
        }
    }
}
