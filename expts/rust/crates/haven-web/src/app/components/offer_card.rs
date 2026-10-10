use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Slot-based offer card component adhering to Haven's fixed alignment rules.
#[component]
pub async fn offer_card(
    #[into] title: String,
    #[into] href: String,
    #[into] price_text: String,
    #[into] description: String,
    #[default] edit_href: Option<String>,
    #[default] delete_action: Option<String>,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <li class="offer-card">
            <div class="offer-header">
                <h3 class="offer-title">
                    <a href=(href)>(title)</a>
                </h3>
                <span class="offer-price">(price_text)</span>
            </div>
            <p class="offer-description">(description)</p>
            if edit_href.is_some() || delete_action.is_some() {
                <div class="offer-actions">
                    if let Some(edit_url) = edit_href {
                        <a href=(edit_url) class="btn btn-secondary">"Edit"</a>
                    }
                    if let Some(del_url) = delete_action {
                        <a href=(del_url) class="btn btn-danger">"Delete"</a>
                    }
                </div>
            }
        </li>
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
    async fn offer_card_renders_slots_and_actions() {
        let cx = Cx::default();
        let __cx = &cx;
        let card = view! {
            offer_card(
                title: "Vintage Leather Jacket",
                href: "/offers/vintage-leather-jacket-abc",
                price_text: "50 USD",
                description: "A well-preserved leather jacket in great condition.",
                edit_href: Some("/offers/uuid/edit".to_string()),
                delete_action: Some("/offers/uuid/delete".to_string()),
            )
        };

        let html = card.single().await.unwrap().render(__cx);
        assert!(html.contains("Vintage Leather Jacket"));
        assert!(html.contains("href=\"/offers/vintage-leather-jacket-abc\""));
        assert!(html.contains("50 USD"));
        assert!(html.contains("A well-preserved leather jacket"));
        assert!(html.contains("href=\"/offers/uuid/edit\""));
        assert!(html.contains("href=\"/offers/uuid/delete\""));
    }
}
