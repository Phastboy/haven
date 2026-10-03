#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    view::view,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn edit_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    Ok(view! {
        <div class="edit-offer">
            <h1>"Edit Offer"</h1>
        </div>
    })
}

#[topcoat::router::page(POST)]
pub async fn update_offer(cx: &Cx) -> TopcoatResult<()> {
    Err(redirect("/offers").into())
}
