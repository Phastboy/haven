//! List offers owned by the authenticated user.

use crate::app::components::button::{ButtonVariant, button_link};
use crate::app::components::empty_state::empty_state;
use crate::app::components::offer_card::offer_card;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

/// Renders the list of offers owned by the current authenticated user at `/offers/manage`.
#[topcoat::router::page("/offers/manage")]
pub async fn manage_list(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let offers = crate::app::state::registry(cx)
        .offers()
        .find_by_user(user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Ok(view! {
        <div class="offers-manage">
            <meta name="robots" content="noindex" />
            <div class="page-header-row">
                <div>
                    <h1>"Your Offers"</h1>
                    <p class="text-muted">"Manage the offers you have listed on Haven."</p>
                </div>
                button_link(
                    href: "/offers/manage/new",
                    text: "New Offer",
                    variant: ButtonVariant::Primary,
                )
            </div>
            if offers.is_empty() {
                empty_state(
                    title: "You haven't listed any offers yet",
                    message: "Create an offer to make what you provide available for others to see.",
                    action_href: Some("/offers/manage/new".to_string()),
                    action_label: Some("Create an Offer".to_string()),
                )
            } else {
                <ul class="offer-list" role="list">
                    for offer in offers {
                        offer_card(
                            title: offer.title.clone(),
                            href: format!("/offers/manage/{}", offer.id),
                            price_text: if offer.price.as_i32() == 0 {
                                "Free".to_string()
                            } else {
                                format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
                            },
                            description: offer.description.clone(),
                            edit_href: Some(format!("/offers/manage/{}/edit", offer.id)),
                            delete_action: Some(format!("/offers/manage/{}/delete", offer.id)),
                        )
                    }
                </ul>
            }
        </div>
    })
}
