use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::field::{text_field, textarea_field};
use topcoat::router::error::bad_request;
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

#[topcoat::router::page]
pub async fn edit_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer =
        crate::app::auth::guard::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    Ok(view! {
        <div class="edit-offer">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <h1>"Edit Offer"</h1>
                <p class="text-muted">"Update your offer's details."</p>
            </div>
            <form method="post" action=(format!("/offers/manage/{}/edit", offer.id)) class="form-stack">
                text_field(
                    label: "Title",
                    name: "title",
                    value: Some(offer.title),
                    required: true,
                    minlength: Some(3),
                    maxlength: Some(100),
                )
                textarea_field(
                    label: "Description",
                    name: "description",
                    value: Some(offer.description),
                    required: true,
                    minlength: Some(10),
                    maxlength: Some(2000),
                )
                text_field(
                    label: "Price (minor units)",
                    name: "price",
                    field_type: "number",
                    value: Some(offer.price.as_i32().to_string()),
                    required: true,
                    min: Some(0),
                    placeholder: Some("0 for Free".to_string()),
                    help: Some("Enter 0 for a free offer.".to_string()),
                )
                text_field(
                    label: "Currency",
                    name: "currency",
                    value: Some(offer.currency.as_str().to_string()),
                    placeholder: Some("NGN (required if price > 0)".to_string()),
                    maxlength: Some(3),
                )
                <div class="form-actions">
                    button(
                        text: "Save Changes",
                        variant: ButtonVariant::Primary,
                    )
                    button_link(
                        href: format!("/offers/manage/{}", offer.id),
                        text: "Cancel",
                        variant: ButtonVariant::Secondary,
                    )
                </div>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct EditOfferForm {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
}

#[topcoat::router::page(POST)]
pub async fn update_offer(
    cx: &Cx,
    form: Option<topcoat::router::content::Form<EditOfferForm>>,
) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId(id_uuid);

    // Ownership and auth check
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let Some(form) = form else {
        return Err(bad_request("Missing form body").into());
    };

    let patch = super::super::super::fields::parse_for_patch(
        form.0.title.as_deref(),
        form.0.description.as_deref(),
        form.0.price.as_deref(),
        form.0.currency.as_deref(),
    )?;

    let domain_update = haven_domain::offer::UpdateOffer {
        title: patch.title,
        description: patch.description,
        price: patch.price,
        currency: patch.currency,
    };

    domain_update
        .validate()
        .map_err(|e| bad_request(e.to_string()))?;

    crate::app::state::registry(cx)
        .offers()
        .update(offer_id, user.id, &domain_update)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other(format!("/offers/manage/{offer_id}")).into())
}
