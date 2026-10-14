//! Profile avatar deletion route at `/profile/avatar/delete`.

use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other};

#[topcoat::router::page([POST, DELETE])]
pub async fn delete_avatar(cx: &Cx) -> TopcoatResult<()> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    if user.avatar_public_id.is_some() {
        let registry = crate::app::state::registry(cx);
        registry
            .users()
            .update_avatar(user.id, None)
            .await
            .map_err(crate::app::state::map_repo_err)?;
    }

    Err(see_other("/profile/edit?avatar_deleted=1").into())
}
