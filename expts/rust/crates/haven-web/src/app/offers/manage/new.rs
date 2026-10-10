use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::field::{text_field, textarea_field};
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

#[derive(Default)]
struct NewOfferFormFields {
    title: Option<String>,
    description: Option<String>,
    price: Option<String>,
    currency: Option<String>,
}

struct NewOfferFormView {
    idempotency_key: String,
    fields: NewOfferFormFields,
    error_banner: Option<String>,
    status_code: Option<http::StatusCode>,
}

fn render_new_offer_form(cx: &Cx, view_data: NewOfferFormView) -> impl topcoat::view::View {
    let __cx = cx;
    let idempotency_key = view_data.idempotency_key;
    let fields = view_data.fields;
    view! {
        <div class="new-offer">
            <meta name="robots" content="noindex" />
            if let Some(status) = view_data.status_code {
                (status)
            }
            <div class="page-header">
                <h1>"New Offer"</h1>
                <p class="text-muted">"Create a new offer to list on Haven."</p>
            </div>
            if let Some(err) = view_data.error_banner {
                <div class="form-error-banner" role="alert">
                    <p>(err)</p>
                </div>
            }
            <form method="post" action="/offers/manage/new" class="form-stack">
                <input type="hidden" name="idempotency_key" value=(idempotency_key) />
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
                        text: "Create Offer",
                        variant: ButtonVariant::Primary,
                    )
                    button_link(
                        href: "/offers/manage",
                        text: "Cancel",
                        variant: ButtonVariant::Secondary,
                    )
                </div>
            </form>
        </div>
    }
}

#[topcoat::router::page]
pub async fn new_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let _ = crate::app::auth::guard::require_owner_auth(cx).await?;
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    Ok(render_new_offer_form(
        cx,
        NewOfferFormView {
            idempotency_key,
            fields: NewOfferFormFields::default(),
            error_banner: None,
            status_code: None,
        },
    ))
}

#[derive(serde::Deserialize)]
pub struct NewOfferForm {
    pub idempotency_key: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
}

#[topcoat::router::page(POST)]
pub async fn create_offer(
    cx: &Cx,
    form: Option<topcoat::router::content::Form<NewOfferForm>>,
) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let Some(form) = form else {
        return Ok(render_new_offer_form(
            cx,
            NewOfferFormView {
                idempotency_key: uuid::Uuid::new_v4().to_string(),
                fields: NewOfferFormFields::default(),
                error_banner: Some("Missing form submission.".to_string()),
                status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
            },
        ));
    };

    let f = form.0;
    let raw_key = f.idempotency_key.to_string();
    let fields = NewOfferFormFields {
        title: Some(f.title),
        description: f.description,
        price: f.price,
        currency: f.currency,
    };

    let limiter = crate::rate_limit::create_offer_limiter(cx);
    crate::rate_limit::enforce(limiter, &user.id.to_string())?;

    let (price, currency) = match super::super::fields::parse_for_create(
        fields.price.as_deref(),
        fields.currency.as_deref(),
    ) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Ok(render_new_offer_form(
                cx,
                NewOfferFormView {
                    idempotency_key: raw_key,
                    fields,
                    error_banner: Some(e.to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }
    };

    let create_req = haven_domain::offer::CreateOffer {
        title: fields.title.clone().unwrap_or_default(),
        description: fields.description.clone().unwrap_or_default(),
        price,
        currency,
    };

    if let Err(e) = create_req.validate() {
        return Ok(render_new_offer_form(
            cx,
            NewOfferFormView {
                idempotency_key: raw_key,
                fields,
                error_banner: Some(e.to_string()),
                status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
            },
        ));
    }

    let offer = crate::app::state::registry(cx)
        .offers()
        .create(
            user.id,
            haven_domain::ports::IdempotencyKey::new(f.idempotency_key),
            &create_req,
        )
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other(format!("/offers/manage/{}", offer.id)).into())
}
