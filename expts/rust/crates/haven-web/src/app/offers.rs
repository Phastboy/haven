#![allow(unused_variables)]
pub mod new;
pub mod id;

use topcoat::{
    context::Cx,
    view::view,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn index(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    
    Ok(view! {
        <div class="offers">
            <h1>"Offers"</h1>
            <a href="/offers/new">"New Offer"</a>
        </div>
    })
}
