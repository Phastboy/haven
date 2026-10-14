use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Slot-based offer card component adhering to Haven's fixed alignment rules.
#[component]
#[allow(
    clippy::too_many_arguments,
    reason = "Component props macro generates multi-argument constructor"
)]
pub async fn offer_card(
    #[into] title: String,
    #[into] href: String,
    #[into] price_text: String,
    #[into] description: String,
    #[default] maker_name: Option<String>,
    #[default] maker_href: Option<String>,
    #[default] edit_href: Option<String>,
    #[default] delete_action: Option<String>,
    #[default] cover_image_url: Option<String>,
    #[default] cover_image_srcset: Option<String>,
) -> TopcoatResult<impl View> {
    let dialog_id = delete_action
        .as_ref()
        .map(|act| format!("confirm-del-{}", act.trim_matches('/').replace('/', "-")));

    Ok(view! {
        <li class="offer-card">
            <div class="offer-card-media">
                if let Some(src) = cover_image_url {
                    <img
                        src=(src)
                        srcset=(cover_image_srcset.unwrap_or_default())
                        alt=""
                        class="offer-card-image"
                        width="320"
                        height="240"
                        loading="lazy"
                    />
                } else {
                    <div class="offer-card-placeholder" aria-hidden="true" />
                }
            </div>
            <div class="offer-header">
                <h3 class="offer-title">
                    <a href=(href)>(title.clone())</a>
                </h3>
                <span class="offer-price">(price_text)</span>
            </div>
            if let (Some(name), Some(url)) = (maker_name, maker_href) {
                <div class="offer-maker">
                    <span class="text-muted">"by "</span>
                    <a href=(url) class="maker-link">(name)</a>
                </div>
            }
            <p class="offer-description">(description)</p>
            if edit_href.is_some() || delete_action.is_some() {
                <div class="offer-actions">
                    if let Some(edit_url) = edit_href {
                        <a href=(edit_url) class="btn btn-secondary">"Edit"</a>
                    }
                    if let (Some(del_url), Some(dlg_id)) = (delete_action.as_ref(), dialog_id.as_ref()) {
                        <button
                            type="button"
                            popovertarget=(dlg_id.clone())
                            class="btn btn-danger"
                        >
                            "Delete"
                        </button>
                        <dialog id=(dlg_id.clone()) popover="auto" class="confirm-dialog">
                            <div class="confirm-dialog-content">
                                <h2 class="dialog-title">"Delete Offer"</h2>
                                <p class="dialog-message">
                                    "Are you sure you want to delete " <strong>(&title)</strong> "? This action cannot be undone."
                                </p>
                                <div class="dialog-actions">
                                    <button
                                        type="button"
                                        popovertarget=(dlg_id.clone())
                                        popovertargetaction="hide"
                                        class="btn btn-secondary"
                                    >
                                        "Cancel"
                                    </button>
                                    <form method="post" action=(del_url) class="dialog-confirm-form">
                                        <button type="submit" class="btn btn-danger">
                                            "Confirm Delete"
                                        </button>
                                    </form>
                                </div>
                            </div>
                        </dialog>
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
                maker_name: Some("Alex Maker".to_string()),
                maker_href: Some("/profiles/alex".to_string()),
                edit_href: Some("/profile/offers/uuid/edit".to_string()),
                delete_action: Some("/profile/offers/uuid".to_string()),
            )
        };

        let html = card.single().await.unwrap().render(__cx);
        assert!(html.contains("Vintage Leather Jacket"));
        assert!(html.contains("href=\"/offers/vintage-leather-jacket-abc\""));
        assert!(html.contains("50 USD"));
        assert!(html.contains("A well-preserved leather jacket"));
        assert!(html.contains("Alex Maker"));
        assert!(html.contains("href=\"/profiles/alex\""));
        assert!(html.contains("href=\"/profile/offers/uuid/edit\""));
        assert!(html.contains("popovertarget=\"confirm-del-profile-offers-uuid\""));
        assert!(html.contains("<dialog id=\"confirm-del-profile-offers-uuid\" popover=\"auto\""));
        assert!(html.contains("action=\"/profile/offers/uuid\""));
        assert!(html.contains("class=\"offer-card-placeholder\""));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn offer_card_renders_cover_image() {
        let cx = Cx::default();
        let __cx = &cx;
        let card = view! {
            offer_card(
                title: "Handmade Mug",
                href: "/offers/handmade-mug-123",
                price_text: "15 USD",
                description: "Ceramic coffee mug.",
                cover_image_url: Some("https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg".to_string()),
                cover_image_srcset: Some("https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg 1x, https://res.cloudinary.com/haven/image/private/t_card_2x/offers/123/cover.jpg 2x".to_string()),
            )
        };

        let html = card.single().await.unwrap().render(__cx);
        assert!(html.contains("class=\"offer-card-image\""));
        assert!(html.contains("src=\"https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg\""));
        assert!(html.contains("srcset=\"https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg 1x, https://res.cloudinary.com/haven/image/private/t_card_2x/offers/123/cover.jpg 2x\""));
        assert!(html.contains("alt=\"\""));
        assert!(!html.contains("offer-card-placeholder"));
    }
}
