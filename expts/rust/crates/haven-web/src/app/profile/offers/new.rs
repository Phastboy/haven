#![allow(
    clippy::too_many_lines,
    reason = "Multipart offer creation with validation, form error rendering, and image upload"
)]

use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::category_select::category_select;
use crate::app::components::field::{text_field, textarea_field};
use haven_domain::image::{
    ImageDimensions, ImagePublicId, MAX_IMAGE_BYTES, validate_image_magic_bytes,
};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::multipart::Multipart, error::see_other},
    view::view,
};

#[derive(Default)]
struct NewOfferFormFields {
    title: Option<String>,
    description: Option<String>,
    price: Option<String>,
    currency: Option<String>,
    category: Option<String>,
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
    let category_error = view_data.status_code.and_then(|_| {
        crate::app::offers::fields::parse_category(fields.category.as_deref())
            .err()
            .map(|error| error.to_string())
    });
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
            <form method="post" action="/profile/offers/new" enctype="multipart/form-data" class="form-stack">
                <input type="hidden" name="idempotency_key" value=(idempotency_key) />
                text_field(
                    label: "Title",
                    name: "title",
                    value: fields.title,
                    required: true,
                    minlength: Some(3),
                    maxlength: Some(50),
                )
                category_select(
                    id: "offer-category",
                    selected: fields.category,
                    required: true,
                    error: category_error,
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
                <div class="field">
                    <label for="new-offer-photo" class="field-label">"Cover Photo (optional)"</label>
                    <input
                        type="file"
                        id="new-offer-photo"
                        name="photo"
                        accept="image/jpeg,image/png,image/webp"
                        class="file-input"
                    />
                    <p class="field-help text-muted">"JPEG, PNG, or WebP up to 5 MB. Offers without a photo stay private to you."</p>
                </div>
                <div class="form-actions">
                    button(
                        text: "Create Offer",
                        variant: ButtonVariant::Primary,
                    )
                    button_link(
                        href: "/profile/offers",
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

#[topcoat::router::page(POST)]
pub async fn create_offer(
    cx: &Cx,
    multipart: Option<Multipart>,
) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let Some(mut mp) = multipart else {
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

    let mut key: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut price: Option<String> = None;
    let mut currency: Option<String> = None;
    let mut category: Option<String> = None;
    let mut photo_bytes: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = mp.next_field().await {
        match field.name() {
            Some("idempotency_key") => key = field.text().await.ok(),
            Some("title") => title = field.text().await.ok(),
            Some("description") => description = field.text().await.ok(),
            Some("price") => price = field.text().await.ok(),
            Some("currency") => currency = field.text().await.ok(),
            Some("category") => category = field.text().await.ok(),
            Some("photo") => {
                if let Ok(bytes) = field.bytes().await {
                    photo_bytes = (!bytes.is_empty()).then(|| bytes.to_vec());
                }
            }
            _ => {}
        }
    }

    let raw_key = key.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let fields = NewOfferFormFields {
        title,
        description,
        price,
        currency,
        category,
    };

    let category = match crate::app::offers::fields::parse_category(fields.category.as_deref()) {
        Ok(category) => category,
        Err(error) => {
            return Ok(render_new_offer_form(
                cx,
                NewOfferFormView {
                    idempotency_key: raw_key,
                    fields,
                    error_banner: Some(error.to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }
    };

    if let Some(bytes) = photo_bytes.as_ref() {
        if bytes.len() > MAX_IMAGE_BYTES {
            return Ok(render_new_offer_form(
                cx,
                NewOfferFormView {
                    idempotency_key: raw_key,
                    fields,
                    error_banner: Some("Photo is too large. Maximum size is 5 MB.".to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }

        if validate_image_magic_bytes(bytes).is_err() {
            return Ok(render_new_offer_form(
                cx,
                NewOfferFormView {
                    idempotency_key: raw_key,
                    fields,
                    error_banner: Some(
                        "Unsupported photo format. Only JPEG, PNG, and WebP are allowed (no SVG)."
                            .to_string(),
                    ),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }
    }

    let (price, currency) = match crate::app::offers::fields::parse_for_create(
        fields.price.as_deref(),
        fields.currency.as_deref(),
    ) {
        Ok(res) => res,
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

    let title_str = fields.title.clone().unwrap_or_default();
    let desc_str = fields.description.clone().unwrap_or_default();

    let create_req = haven_domain::offer::CreateOffer {
        category,
        title: title_str,
        description: desc_str,
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

    let key_uuid = uuid::Uuid::parse_str(&raw_key).unwrap_or_else(|_| uuid::Uuid::new_v4());

    let status = crate::app::state::registry(cx)
        .offers()
        .create(
            user.id,
            haven_domain::ports::IdempotencyKey::new(key_uuid),
            &create_req,
        )
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let is_new = status.is_new();
    let offer = status.into_inner();

    if let (true, Some(bytes)) = (is_new, photo_bytes.as_ref()) {
        let storage = crate::app::state::image_storage(cx);
        let folder = format!("offers/{}", offer.id);
        let suffix = uuid::Uuid::new_v4().to_string();
        if let Ok(stored) = storage.upload(&folder, &suffix, bytes).await {
            let dimensions = ImageDimensions::new(stored.width, stored.height).unwrap_or_default();
            if let Ok(public_id) = ImagePublicId::parse(&stored.public_id) {
                match crate::app::state::registry(cx)
                    .offer_images()
                    .add_image(offer.id, user.id, &public_id, dimensions)
                    .await
                {
                    Ok(_) | Err(_) => {}
                }
            }
        }
    }

    Err(see_other(format!("/profile/offers/{}", offer.id)).into())
}
