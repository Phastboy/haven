//! Public offers feed page and query parameters.

use crate::app::components::empty_state::empty_state;
use crate::app::components::offer_card::offer_card;
use crate::app::components::pagination::pagination_nav;
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

/// Keyset pagination query parameter for the public offers feed.
#[topcoat::router::query_params]
pub struct FeedQuery {
    /// Opaque cursor string marking the offer to list after.
    pub after: Option<String>,
}

/// Serves the public offers feed page at `/offers`.
#[topcoat::router::page("/offers")]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let query = topcoat::router::query_params::<FeedQuery>(cx).ok();
    let after_cursor = query
        .and_then(|q| q.after.as_deref())
        .and_then(|raw| haven_domain::offer::OfferCursor::decode(raw).ok());

    let repo = crate::app::state::registry(cx).offers();
    let page = repo
        .list_public(after_cursor.as_ref(), 20)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let next_href = page.next_cursor.map(|c| format!("/offers?after={c}"));

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);

    Ok(view! {
        <div class="offers-feed">
            <div class="page-header">
                <h1>"Available Offers"</h1>
                <p class="text-muted">"Browse offers available right now on Haven."</p>
            </div>
            if page.items.is_empty() {
                empty_state(
                    title: "No offers available yet",
                    message: "There are no offers posted on Haven right now. Check back soon or create the first one.",
                    action_href: Some("/profile/offers/new".to_string()),
                    action_label: Some("Create an Offer".to_string()),
                )
            } else {
                <ul class="offer-list" role="list">
                    for item in page.items {
                        let cover = item.offer.cover_image();
                        let cover_url = cover.map(|img| crate::app::components::image::card_url(cloud_name, img.public_id.as_str()));
                        let cover_srcset = cover.map(|img| crate::app::components::image::card_srcset(cloud_name, img.public_id.as_str()));
                        offer_card(
                            title: item.offer.title.clone(),
                            href: format!("/offers/{}", item.offer.slug),
                            price_text: if item.offer.price.as_i32() == 0 {
                                "Free".to_string()
                            } else {
                                format!("{} {}", item.offer.price.as_i32(), item.offer.currency.as_str())
                            },
                            description: item.offer.description.clone(),
                            maker_name: Some(item.maker_display_name.as_str().to_string()),
                            maker_href: Some(format!("/profiles/{}", item.maker_handle.as_str())),
                            cover_image_url: cover_url,
                            cover_image_srcset: cover_srcset,
                        )
                    }
                </ul>
                if next_href.is_some() {
                    pagination_nav(
                        prev_href: None,
                        next_href: next_href,
                    )
                }
            }
        </div>
    })
}
