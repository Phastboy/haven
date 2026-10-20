//! Thread message and offer reference components.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

#[component]
pub async fn offer_tag_reference(
    #[into] title: String,
    #[into] slug: String,
    #[default] cover_url: Option<String>,
    #[default] is_available: bool,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="message-offer-tag">
            if is_available {
                <a href=(format!("/offers/{slug}")) class="offer-tag-link">
                    if let Some(url) = cover_url {
                        <img src=(url) alt="" class="offer-tag-thumb" width="48" height="48" />
                    }
                    <span class="offer-tag-title">(title)</span>
                </a>
            } else {
                <span class="offer-tag-unavailable">"This offer is no longer available"</span>
            }
        </div>
    })
}

#[component]
pub async fn message_item(
    #[into] id: String,
    #[default] is_own: bool,
    #[into] sender_name: String,
    #[into] body: String,
    #[into] time_str: String,
    #[default] offer_tag_title: Option<String>,
    #[default] offer_tag_slug: Option<String>,
    #[default] offer_tag_cover: Option<String>,
    #[default] offer_tag_available: bool,
) -> TopcoatResult<impl View> {
    let id_attr = format!("msg-{id}");

    Ok(view! {
        <div class=(if is_own { "message-row own-message" } else { "message-row other-message" }) id=(id_attr)>
            <div class="message-meta">
                <span class="message-sender">(sender_name)</span>
                <span class="message-time">(time_str)</span>
            </div>
            if let (Some(title), Some(slug)) = (offer_tag_title, offer_tag_slug) {
                offer_tag_reference(
                    title: title,
                    slug: slug,
                    cover_url: offer_tag_cover,
                    is_available: offer_tag_available,
                )
            }
            <div class="message-body">
                <p>(body)</p>
            </div>
        </div>
    })
}
