#![allow(unused_variables)]
pub mod edit;
pub mod delete;

use topcoat::{
    context::Cx,
    view::view,
    Result as TopcoatResult,
};

use topcoat::router::error::bad_request;
topcoat::router::path_param!(id: uuid::Uuid, error = bad_request);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer = crate::cx_helpers::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;
    
    Ok(view! {
        <div class="view-offer">
            <h1>(offer.title)</h1>
            <p>(offer.description.unwrap_or_default())</p>
            <a href=(format!("/offers/{}/edit", offer.id)) class="button">"Edit"</a>
            <form method="post" action=(format!("/offers/{}/delete", offer.id))>
                <button type="submit" class="button">"Delete"</button>
            </form>
        </div>
    })
}
