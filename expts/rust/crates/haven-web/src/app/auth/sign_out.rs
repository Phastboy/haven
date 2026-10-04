
use topcoat::{
    context::Cx,
    router::error::see_other,
    Result as TopcoatResult,
};

#[topcoat::router::page(POST)]
pub async fn sign_out(cx: &Cx) -> TopcoatResult<()> {
    if let Some(hash) = topcoat::session::stop(cx).await? {
        let hash_hex = crate::cx_helpers::token_hash_hex(&hash);
        let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
        let registry = crate::cx_helpers::registry(cx);
        let map_err = crate::cx_helpers::map_repo_err;
        if let Some(session) = registry.sessions().find_by_token_hash(&hashed_token).await.map_err(map_err)? {
            registry.sessions().delete(session.id).await.map_err(map_err)?;
        }
    }
    Err(see_other("/").into())
}
