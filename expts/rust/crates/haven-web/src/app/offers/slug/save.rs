//! Save or unsave a public offer.

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::Form, error::see_other},
};

#[derive(serde::Deserialize)]
pub struct SaveForm {
    pub action: String,
}

#[topcoat::router::page(POST)]
pub async fn toggle_save(cx: &Cx, form: Form<SaveForm>) -> TopcoatResult<()> {
    let slug_raw: &str = topcoat::router::path_param::<super::Slug>(cx);
    let slug = haven_domain::offer::OfferSlug::parse(slug_raw)
        .map_err(|_| topcoat::router::error::not_found())?;
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let registry = crate::app::state::registry(cx);
    let offer = registry
        .offers()
        .find_by_slug(&slug)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .filter(haven_domain::offer::Offer::is_public)
        .ok_or_else(topcoat::router::error::not_found)?;
    match form.0.action.as_str() {
        "save" => registry.saved_offers().save(user.id, offer.id).await,
        "unsave" => registry.saved_offers().unsave(user.id, offer.id).await,
        _ => return Err(topcoat::router::error::bad_request("invalid save action").into()),
    }
    .map_err(crate::app::state::map_repo_err)?;
    Err(see_other(format!("/offers/{}", slug.as_str())).into())
}
