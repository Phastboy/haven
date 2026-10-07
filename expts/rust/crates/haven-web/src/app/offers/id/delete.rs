use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other};

#[topcoat::router::page(POST)]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId(id_uuid);

    // Authorization check
    let _offer = crate::cx_helpers::owned_offer(cx, offer_id).await?;

    let user = crate::cx_helpers::require_auth(cx).await?;
    crate::cx_helpers::registry(cx)
        .offers()
        .delete(offer_id, user.id)
        .await
        .map_err(crate::cx_helpers::map_repo_err)?;

    Err(see_other("/offers/new").into())
}
