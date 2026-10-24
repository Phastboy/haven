//! Follow or unfollow the person shown on the public profile.
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::Form, error::see_other},
};
#[derive(serde::Deserialize)]
pub struct FollowForm {
    pub action: String,
}

#[topcoat::router::page(POST)]
pub async fn toggle(cx: &Cx, form: Form<FollowForm>) -> TopcoatResult<()> {
    let raw: &str = topcoat::router::path_param::<super::Handle>(cx);
    let handle = haven_domain::user::ProfileHandle::parse(raw)
        .map_err(|_| topcoat::router::error::not_found())?;
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let registry = crate::app::state::registry(cx);
    let person = registry
        .users()
        .find_by_handle(&handle)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;
    match form.0.action.as_str() {
        "follow" => registry.follows().follow(user.id, person.id).await,
        "unfollow" => registry.follows().unfollow(user.id, person.id).await,
        _ => return Err(topcoat::router::error::bad_request("invalid follow action").into()),
    }
    .map_err(crate::app::state::map_repo_err)?;
    Err(see_other(format!("/profiles/{}", handle.as_str())).into())
}
