pub mod delete;
pub mod edit;

use crate::app::components::button::{ButtonVariant, button, button_link};
use topcoat::{Result as TopcoatResult, context::Cx, view::view};

topcoat::router::module_param!(id: uuid::Uuid, error = not_found);

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer =
        crate::app::auth::guard::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    let price_text = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };

    Ok(view! {
        <div class="offer-detail">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <a href="/offers/manage" class="text-muted back-link">"← Back to your offers"</a>
                <div class="offer-detail-header">
                    <h1 class="offer-detail-title">(offer.title)</h1>
                    <div class="offer-detail-price">(price_text)</div>
                </div>
            </div>
            <div class="offer-detail-body">
                <p>(offer.description)</p>
            </div>
            <div class="owner-actions">
                button_link(
                    href: format!("/offers/manage/{}/edit", offer.id),
                    text: "Edit",
                    variant: ButtonVariant::Primary,
                )
                button_link(
                    href: format!("/offers/{}", offer.slug),
                    text: "View Public Page",
                    variant: ButtonVariant::Secondary,
                )
                <form method="post" action=(format!("/offers/manage/{}/delete", offer.id)) class="inline-form">
                    button(
                        text: "Delete",
                        variant: ButtonVariant::Danger,
                    )
                </form>
            </div>
        </div>
    })
}
