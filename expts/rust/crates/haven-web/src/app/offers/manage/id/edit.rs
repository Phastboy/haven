use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::field::{text_field, textarea_field};
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

#[derive(Default)]
struct EditOfferFormFields {
    title: Option<String>,
    description: Option<String>,
    price: Option<String>,
    currency: Option<String>,
}

struct EditOfferFormView {
    offer_id: haven_domain::offer::OfferId,
    fields: EditOfferFormFields,
    error_banner: Option<String>,
    status_code: Option<http::StatusCode>,
}

fn render_edit_offer_form(cx: &Cx, view_data: EditOfferFormView) -> impl topcoat::view::View {
    let __cx = cx;
    let offer_id = view_data.offer_id;
    let fields = view_data.fields;
    view! {
        <div class="edit-offer">
            <meta name="robots" content="noindex" />
            if let Some(status) = view_data.status_code {
                (status)
            }
            <div class="page-header">
                <h1>"Edit Offer"</h1>
                <p class="text-muted">"Update your offer's details."</p>
            </div>
            if let Some(err) = view_data.error_banner {
                <div class="form-error-banner" role="alert">
                    <p>(err)</p>
                </div>
            }
            <form method="post" action=(format!("/offers/manage/{}/edit", offer_id)) class="form-stack">
                text_field(
                    label: "Title",
                    name: "title",
                    value: fields.title,
                    required: true,
                    minlength: Some(3),
                    maxlength: Some(100),
                )
                textarea_field(
                    label: "Description",
                    name: "description",
                    value: fields.description,
                    required: true,
                    minlength: Some(10),
                    maxlength: Some(2000),
                )
                text_field(
                    label: "Price (minor units)",
                    name: "price",
                    field_type: "number",
                    value: fields.price,
                    required: true,
                    min: Some(0),
                    placeholder: Some("0 for Free".to_string()),
                    help: Some("Enter 0 for a free offer.".to_string()),
                )
                text_field(
                    label: "Currency",
                    name: "currency",
                    value: fields.currency,
                    placeholder: Some("NGN (required if price > 0)".to_string()),
                    maxlength: Some(3),
                )
                <div class="form-actions">
                    button(
                        text: "Save Changes",
                        variant: ButtonVariant::Primary,
                    )
                    button_link(
                        href: format!("/offers/manage/{}", offer_id),
                        text: "Cancel",
                        variant: ButtonVariant::Secondary,
                    )
                </div>
            </form>
        </div>
    }
}

/// Renders the edit offer form for an existing offer owned by the current user.
#[topcoat::router::page]
pub async fn edit_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer =
        crate::app::auth::guard::owned_offer(cx, haven_domain::offer::OfferId::from_uuid(id_uuid))
            .await?;

    Ok(render_edit_offer_form(
        cx,
        EditOfferFormView {
            offer_id: offer.id,
            fields: EditOfferFormFields {
                title: Some(offer.title),
                description: Some(offer.description),
                price: Some(offer.price.as_i32().to_string()),
                currency: Some(offer.currency.as_str().to_string()),
            },
            error_banner: None,
            status_code: None,
        },
    ))
}

/// Form payload for submitting updates to an existing offer.
#[derive(serde::Deserialize)]
pub struct EditOfferForm {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
}

/// Processes an offer edit submission, verifying ownership prior to form validation.
#[topcoat::router::page(POST)]
pub async fn update_offer(
    cx: &Cx,
    form: Option<topcoat::router::content::Form<EditOfferForm>>,
) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    // Ownership and auth check
    crate::app::auth::guard::owned_offer(cx, offer_id).await?;
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let Some(form) = form else {
        return Ok(render_edit_offer_form(
            cx,
            EditOfferFormView {
                offer_id,
                fields: EditOfferFormFields::default(),
                error_banner: Some("Missing form submission.".to_string()),
                status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
            },
        ));
    };

    let f = form.0;
    let fields = EditOfferFormFields {
        title: f.title,
        description: f.description,
        price: f.price,
        currency: f.currency,
    };

    let patch = match super::super::super::fields::parse_for_patch(
        super::super::super::fields::RawOfferPatch {
            title: fields.title.as_deref(),
            description: fields.description.as_deref(),
            price: fields.price.as_deref(),
            currency: fields.currency.as_deref(),
        },
    ) {
        Ok(p) => p,
        Err(e) => {
            return Ok(render_edit_offer_form(
                cx,
                EditOfferFormView {
                    offer_id,
                    fields,
                    error_banner: Some(e.to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }
    };

    let domain_update = haven_domain::offer::UpdateOffer {
        title: patch.title,
        description: patch.description,
        price: patch.price,
        currency: patch.currency,
    };

    if let Err(e) = domain_update.validate() {
        return Ok(render_edit_offer_form(
            cx,
            EditOfferFormView {
                offer_id,
                fields,
                error_banner: Some(e.to_string()),
                status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
            },
        ));
    }

    crate::app::state::registry(cx)
        .offers()
        .update(offer_id, user.id, &domain_update)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other(format!("/offers/manage/{offer_id}")).into())
}
