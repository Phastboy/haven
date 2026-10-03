#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    view::view,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn new_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    Ok(view! {
        <div class="new-offer">
            <h1>"New Offer"</h1>
            <form method="post" action="/offers/new">
                <button type="submit">"Create"</button>
            </form>
        </div>
    })
}

#[topcoat::router::page(POST)]
pub async fn create_offer(cx: &Cx) -> TopcoatResult<()> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    Err(redirect("/offers").into())
}
