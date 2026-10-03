#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    Result as TopcoatResult,
};

#[topcoat::router::page(POST)]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId(id_uuid);
    
    // Authorization check
    let _offer = crate::cx_helpers::owned_offer(cx, offer_id).await?;
    
    let db = crate::cx_helpers::db(cx);
    haven_db::offers::delete(db, offer_id).await.map_err(topcoat::Error::from)?;
    
    Err(redirect("/offers").into())
}
