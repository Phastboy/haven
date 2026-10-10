use crate::app::components::button::{ButtonVariant, button, button_link};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::error::see_other,
    view::{View, view},
};

#[topcoat::router::page]
pub async fn confirm_delete(cx: &Cx) -> TopcoatResult<impl View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    let offer = crate::app::auth::guard::owned_offer(cx, offer_id).await?;

    Ok(view! {
        <div class="delete-offer-confirm">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <a href=(format!("/offers/manage/{}", offer.id)) class="text-muted back-link">
                    "← Back to offer"
                </a>
                <h1>"Delete Offer"</h1>
                <p class="text-muted">
                    "Are you sure you want to delete " <strong>(offer.title)</strong> "? This action cannot be undone."
                </p>
            </div>
            <form method="post" action=(format!("/offers/manage/{}/delete", offer.id)) class="form-actions">
                button(
                    text: "Confirm Delete",
                    variant: ButtonVariant::Danger,
                )
                button_link(
                    href: format!("/offers/manage/{}", offer.id),
                    text: "Cancel",
                    variant: ButtonVariant::Secondary,
                )
            </form>
        </div>
    })
}

#[topcoat::router::page(POST)]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    // Authorization check
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    crate::app::state::registry(cx)
        .offers()
        .delete(offer_id, user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other("/offers/manage").into())
}
