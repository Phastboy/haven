use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other};

#[topcoat::router::page(POST)]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId(id_uuid);

    // Authorization check
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    crate::app::state::registry(cx)
        .offers()
        .delete(offer_id, user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other("/offers/manage").into())
}
