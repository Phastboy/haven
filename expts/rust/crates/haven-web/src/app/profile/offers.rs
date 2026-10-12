//! Offers management section at `/profile/offers`.

pub mod id;
pub mod new;

use crate::app::components::button::{ButtonVariant, button_link};
use crate::app::components::empty_state::empty_state;
use crate::app::components::offer_card::offer_card;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

/// Renders the list of offers owned by the current authenticated user at `/profile/offers`.
#[topcoat::router::page]
pub async fn profile_offers_page(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let offers = crate::app::state::registry(cx)
        .offers()
        .find_by_user(user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let offers_count = offers.len();

    Ok(view! {
        <div class="profile-offers-section">
            <meta name="robots" content="noindex" />
            <div class="page-header-row">
                <div>
                    <h2 class="section-title">"Your Offers"</h2>
                    <p class="text-muted">
                        if offers_count == 0 {
                            "You haven't listed any offers yet."
                        } else {
                            (format!("You have {offers_count} active offer(s)."))
                        }
                    </p>
                </div>
                button_link(
                    href: "/profile/offers/new",
                    text: "Create an Offer",
                    variant: ButtonVariant::Primary,
                )
            </div>

            if offers.is_empty() {
                empty_state(
                    title: "You haven't listed any offers yet",
                    message: "Create an offer to make what you provide available for others to see.",
                    action_href: Some("/profile/offers/new".to_string()),
                    action_label: Some("Create an Offer".to_string()),
                )
            } else {
                <ul class="offer-list" role="list">
                    for offer in offers {
                        offer_card(
                            title: offer.title.clone(),
                            href: format!("/profile/offers/{}", offer.id),
                            price_text: if offer.price.as_i32() == 0 {
                                "Free".to_string()
                            } else {
                                format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
                            },
                            description: offer.description.clone(),
                            edit_href: Some(format!("/profile/offers/{}/edit", offer.id)),
                            delete_action: Some(format!("/profile/offers/{}", offer.id)),
                        )
                    }
                </ul>
            }
        </div>
    })
}
