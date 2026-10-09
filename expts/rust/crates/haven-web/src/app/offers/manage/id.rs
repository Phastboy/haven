pub mod delete;
pub mod edit;

use topcoat::{Result as TopcoatResult, context::Cx, view::view};

topcoat::router::module_param!(id: uuid::Uuid, error = not_found);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer = crate::cx_helpers::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    let price_text = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };

    Ok(view! {
        <div class="view-offer">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <a href="/offers/manage" class="text-muted back-link">"← Back to your offers"</a>
                <h1 class="offer-title">(offer.title)</h1>
                <div class="offer-price-badge">(price_text)</div>
            </div>
            <div class="offer-body">
                <p class="offer-description-full">(offer.description)</p>
            </div>
            <div class="owner-actions">
                <a href=(format!("/offers/manage/{}/edit", offer.id)) class="btn btn-primary">"Edit"</a>
                <a href=(format!("/offers/{}", offer.slug)) class="btn btn-secondary">"View Public Page"</a>
                <form method="post" action=(format!("/offers/manage/{}/delete", offer.id)) class="inline-form">
                    <button type="submit" class="btn btn-danger">"Delete"</button>
                </form>
            </div>
        </div>
    })
}
