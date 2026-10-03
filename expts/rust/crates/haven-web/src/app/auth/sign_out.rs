
use topcoat::{
    context::Cx,
    router::error::see_other,
    Result as TopcoatResult,
};

#[topcoat::router::page(POST)]
pub async fn sign_out(cx: &Cx) -> TopcoatResult<()> {
    if let Some(hash) = topcoat::session::stop(cx).await.map_err(topcoat::Error::from)? {
        let hash_hex = crate::cx_helpers::token_hash_hex(&hash);
        let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
        let db = crate::cx_helpers::db(cx);
        if let Some(session) = haven_db::sessions::find_by_token_hash(db, &hashed_token).await.map_err(topcoat::Error::from)? {
            haven_db::sessions::delete(db, session.id).await.map_err(topcoat::Error::from)?;
        }
    }
    Err(see_other("/").into())
}
