#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    Result as TopcoatResult,
};

#[topcoat::router::page(POST)]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    Err(redirect("/offers").into())
}
