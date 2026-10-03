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
    let db = crate::cx_helpers::db(cx);
    
    let offers = haven_db::offers::find_by_user(db, user.id).await.map_err(topcoat::Error::from)?;
    
    Ok(view! {
        <div class="offers">
            <h1>"Your Offers"</h1>
            <a href="/offers/new" class="button">"New Offer"</a>
            <ul>
                for offer in offers {
                    <li>
                        <a href=(format!("/offers/{}", offer.id))>
                            (offer.title)
                        </a>
                    </li>
                }
            </ul>
        </div>
    })
}
