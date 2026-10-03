#![allow(unused_variables)]
pub mod edit;
pub mod delete;

use topcoat::{
    context::Cx,
    view::view,
    Result as TopcoatResult,
};

topcoat::router::path_param!(id: uuid::Uuid, error = bad_request);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id = *topcoat::router::path_param::<Id>(cx)?;
    Ok(view! {
        <div class="view-offer">
            <h1>"Offer Details"</h1>
            <a href="/offers/edit">"Edit"</a>
        </div>
    })
}
