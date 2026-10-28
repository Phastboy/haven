use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Computes up to two uppercase initials from a display name (e.g. "Eleanor Vance" -> "EV", "Alex" -> "A").
#[must_use]
pub fn extract_maker_initials(display_name: &str) -> String {
    let words: Vec<&str> = display_name.split_whitespace().collect();
    match words.as_slice() {
        [] => "?".to_string(),
        [single] => single
            .chars()
            .next()
            .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string()),
        [first, .., last] => {
            let f = first.chars().next().unwrap_or('?').to_uppercase();
            let l = last.chars().next().unwrap_or('?').to_uppercase();
            format!("{f}{l}")
        }
    }
}

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
    #[default] is_above_fold: bool,
) -> TopcoatResult<impl View> {
    let card_id = format!("offer-{}", href.rsplit('/').next().unwrap_or("unknown"));
    let dialog_id = delete_action
        .as_ref()
        .map(|act| format!("confirm-del-{}", act.trim_matches('/').replace('/', "-")));

    Ok(view! {
        <li id=(card_id) class="offer-card">
            <div class="offer-card-media">
                <span class="cover-fallback" role="img" aria-label="Image unavailable">"Image unavailable"</span>
                if let Some(src) = cover_image_url {
                    if is_above_fold {
                        <img
                            src=(src)
                            srcset=(cover_image_srcset.unwrap_or_default())
                            alt=""
                            class="offer-card-image" data-cover-image=""
                            width="320"
                            height="240"
                            loading="eager"
                            fetchpriority="high"
                        />
                    } else {
                        <img
                            src=(src)
                            srcset=(cover_image_srcset.unwrap_or_default())
                            alt=""
                            class="offer-card-image" data-cover-image=""
                            width="320"
                            height="240"
                            loading="lazy"
                        />
                    }
                } else {
                    <div class="offer-card-placeholder" aria-hidden="true" />
                }
            </div>
            <div class="offer-card-content">
                if let (Some(name), Some(url)) = (maker_name, maker_href) {
                    let initials = extract_maker_initials(&name);
                    <div class="offer-maker">
                        <a href=(url) class="maker-link offer-maker-link">
                            <span class="offer-maker-badge" aria-hidden="true">(initials)</span>
                            <span class="offer-maker-name">(name)</span>
                        </a>
                    </div>
                }
                <div class="offer-header">
                    <h3 class="offer-title">
                        <a href=(href) class="offer-title-link">(title.clone())</a>
                    </h3>
                </div>
                <div class="offer-price">
                    <span class="offer-price-value">(price_text)</span>
                </div>
                if !description.trim().is_empty() {
                    <p class="offer-description">(description)</p>
                }
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
                            crate::app::components::confirm_dialog::confirm_dialog(
                                id: dlg_id.clone(),
                                title: "Delete Offer".to_string(),
                                message: format!("Are you sure you want to delete \"{}\"? This action cannot be undone.", title),
                                action_url: del_url.clone(),
                                confirm_label: "Confirm Delete".to_string(),
                                is_danger: true,
                            )
                        }
                    </div>
                }
            </div>
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
        assert!(html.contains(
            "src=\"https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg\""
        ));
        assert!(html.contains("srcset=\"https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg 1x, https://res.cloudinary.com/haven/image/private/t_card_2x/offers/123/cover.jpg 2x\""));
        assert!(html.contains("alt=\"\""));
        assert!(html.contains("loading=\"lazy\""));
        assert!(!html.contains("offer-card-placeholder"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn offer_card_renders_above_fold_eager_image() {
        let cx = Cx::default();
        let __cx = &cx;
        let card = view! {
            offer_card(
                title: "Handmade Mug",
                href: "/offers/handmade-mug-123",
                price_text: "Free",
                description: "Ceramic coffee mug.",
                cover_image_url: Some("https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg".to_string()),
                cover_image_srcset: Some("https://res.cloudinary.com/haven/image/private/t_card/offers/123/cover.jpg 1x".to_string()),
                is_above_fold: true,
            )
        };

        let html = card.single().await.unwrap().render(__cx);
        assert!(html.contains("loading=\"eager\""));
        assert!(html.contains("fetchpriority=\"high\""));
    }

    #[test]
    fn test_extract_maker_initials() {
        assert_eq!(extract_maker_initials("Eleanor Vance"), "EV");
        assert_eq!(extract_maker_initials("Marcus Reed"), "MR");
        assert_eq!(extract_maker_initials("Sarah Lin"), "SL");
        assert_eq!(extract_maker_initials("David Walsh"), "DW");
        assert_eq!(extract_maker_initials("Alex"), "A");
        assert_eq!(extract_maker_initials(""), "?");
        assert_eq!(extract_maker_initials("Mary Jane Watson"), "MW");
    }
}
