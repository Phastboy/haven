#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    Result as TopcoatResult,
};

#[topcoat::router::page(POST)]
pub async fn sign_out(cx: &Cx) -> TopcoatResult<()> {
    // Invalidate session in DB and clear cookie
    Err(redirect("/").into())
}
