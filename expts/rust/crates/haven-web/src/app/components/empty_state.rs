use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// An empty state block explaining what belongs here with an optional single next action.
#[component]
pub async fn empty_state(
    #[into] title: String,
    #[into] message: String,
    #[default] action_href: Option<String>,
    #[default] action_label: Option<String>,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="empty-state">
            <h2 class="empty-state-title">(title)</h2>
            <p class="empty-state-text">(message)</p>
            if let (Some(href), Some(label)) = (action_href, action_label) {
                <a href=(href) class="btn btn-primary">
                    (label)
                </a>
            }
        </div>
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
    async fn empty_state_renders_title_and_action() {
        let cx = Cx::default();
        let __cx = &cx;
        let el = view! {
            empty_state(
                title: "No offers yet",
                message: "You haven't listed any offers for others to see.",
                action_href: Some("/offers/new".to_string()),
                action_label: Some("Create an Offer".to_string()),
            )
        };

        let html = el.single().await.unwrap().render(__cx);
        assert!(html.contains("No offers yet"));
        assert!(html.contains("You haven't listed any offers"));
        assert!(html.contains("href=\"/offers/new\""));
        assert!(html.contains("Create an Offer"));
    }
}
