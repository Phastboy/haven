pub mod delete;
pub mod edit;

use topcoat::{Result as TopcoatResult, context::Cx, view::view};

topcoat::router::module_param!(id: uuid::Uuid, error = bad_request);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer = crate::cx_helpers::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    Ok(view! {
        <div class="view-offer">
            <h1>(offer.title)</h1>
            <p>(offer.description)</p>
            <div class="meta">
                "Price: "
                (if offer.price.as_i32() == 0 {
                    "Free".to_string()
                } else {
                    format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
                })
            </div>
            <a href=(format!("/offers/{}/edit", offer.id)) class="button">"Edit"</a>
            <form method="post" action=(format!("/offers/{}/delete", offer.id))>
                <button type="submit" class="button">"Delete"</button>
            </form>
        </div>
    })
}
