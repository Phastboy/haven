pub mod edit;

use crate::app::components::button::{ButtonVariant, button_link};
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

topcoat::router::module_param!(id: uuid::Uuid, error = not_found);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer =
        crate::app::auth::guard::owned_offer(cx, haven_domain::offer::OfferId::from_uuid(id_uuid))
            .await?;

    let price_text = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };

    let public_slug_href = format!("/offers/{}", offer.slug);
    let edit_href = format!("/profile/offers/{}/edit", offer.id);
    let delete_action_href = format!("/profile/offers/{}", offer.id);
    let offer_title = offer.title.clone();

    Ok(view! {
        <div class="offer-detail">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <div class="offer-detail-header">
                    <h1 class="offer-detail-title">(offer.title)</h1>
                    <div class="offer-detail-price">(price_text)</div>
                </div>
            </div>

            <div class="offer-detail-body">
                <p>(offer.description)</p>
            </div>

            <div class="owner-actions">
                button_link(
                    href: edit_href,
                    text: "Edit Offer",
                    variant: ButtonVariant::Primary,
                )
                button_link(
                    href: public_slug_href,
                    text: "View Public Page",
                    variant: ButtonVariant::Secondary,
                )
                <button
                    type="button"
                    popovertarget="delete-offer-dialog"
                    class="btn btn-danger"
                >
                    "Delete Offer"
                </button>
            </div>

            <dialog id="delete-offer-dialog" popover="auto" class="confirm-dialog">
                <div class="confirm-dialog-content">
                    <h2 class="dialog-title">"Delete Offer"</h2>
                    <p class="dialog-message">
                        "Are you sure you want to delete " <strong>(offer_title)</strong> "? This action cannot be undone."
                    </p>
                    <div class="dialog-actions">
                        <button
                            type="button"
                            popovertarget="delete-offer-dialog"
                            popovertargetaction="hide"
                            class="btn btn-secondary"
                        >
                            "Cancel"
                        </button>
                        <form method="post" action=(delete_action_href) class="dialog-confirm-form">
                            <button type="submit" class="btn btn-danger">
                                "Confirm Delete"
                            </button>
                        </form>
                    </div>
                </div>
            </dialog>
        </div>
    })
}

#[topcoat::router::page([POST, DELETE])]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    // Authorization check
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    crate::app::state::registry(cx)
        .offers()
        .delete(offer_id, user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other("/profile/offers").into())
}
