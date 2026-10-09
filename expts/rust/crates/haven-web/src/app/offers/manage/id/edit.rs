use topcoat::router::error::bad_request;
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

#[topcoat::router::page]
pub async fn edit_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer = crate::cx_helpers::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    Ok(view! {
        <div class="edit-offer">
            <meta name="robots" content="noindex" />
            <div class="page-header">
                <h1>"Edit Offer"</h1>
                <p class="text-muted">"Update your offer's details."</p>
            </div>
            <form method="post" action=(format!("/offers/manage/{}/edit", offer.id)) class="form-stack">
                <div class="field-group">
                    <label for="title" class="field-label">"Title"</label>
                    <input id="title" type="text" name="title" value=(offer.title) required="required" minlength="3" maxlength="100" class="field-input" />
                </div>
                <div class="field-group">
                    <label for="description" class="field-label">"Description"</label>
                    <textarea id="description" name="description" required="required" minlength="10" maxlength="2000" rows="5" class="field-textarea">(offer.description)</textarea>
                </div>
                <div class="field-group">
                    <label for="price" class="field-label">"Price (minor units)"</label>
                    <input id="price" type="number" name="price" value=(offer.price.as_i32().to_string()) min="0" required="required" placeholder="0 for Free" class="field-input" />
                    <span class="field-help">"Enter 0 for a free offer."</span>
                </div>
                <div class="field-group">
                    <label for="currency" class="field-label">"Currency"</label>
                    <input id="currency" type="text" name="currency" value=(offer.currency.as_str().to_string()) placeholder="NGN (required if price > 0)" maxlength="3" class="field-input" />
                </div>
                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">"Save Changes"</button>
                    <a href=(format!("/offers/manage/{}", offer.id)) class="btn btn-secondary">"Cancel"</a>
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
    let user = crate::cx_helpers::require_owner_auth(cx).await?;

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

    crate::cx_helpers::registry(cx)
        .offers()
        .update(offer_id, user.id, &domain_update)
        .await
        .map_err(crate::cx_helpers::map_repo_err)?;

    Err(see_other(format!("/offers/manage/{offer_id}")).into())
}
