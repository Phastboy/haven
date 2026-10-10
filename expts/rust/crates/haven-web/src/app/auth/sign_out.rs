use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other};

#[topcoat::router::page(POST)]
pub async fn sign_out(cx: &Cx) -> TopcoatResult<()> {
    if let Some(hash) = topcoat::session::stop(cx).await? {
        let hash_hex = crate::app::auth::guard::token_hash_hex(&hash);
        let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
        let registry = crate::app::state::registry(cx);
        let map_err = crate::app::state::map_repo_err;
        if let Some(session) = registry
            .sessions()
            .find_by_token_hash(&hashed_token)
            .await
            .map_err(map_err)?
        {
            registry
                .sessions()
                .delete(session.id)
                .await
                .map_err(map_err)?;
        }
    }
    Err(see_other("/").into())
}
