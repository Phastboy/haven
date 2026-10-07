use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

use topcoat::router::error::bad_request;

#[topcoat::router::page]
pub async fn edit_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer = crate::cx_helpers::owned_offer(cx, haven_domain::offer::OfferId(id_uuid)).await?;

    Ok(view! {
        <div class="edit-offer">
            <h1>"Edit Offer"</h1>
            <form method="post" action=(format!("/offers/{}/edit", offer.id))>
                <label>
                    "Title:"
                    <input type="text" name="title" value=(offer.title) required="required" />
                </label>
                <label>
                    "Description:"
                    <textarea name="description">(offer.description.unwrap_or_default())</textarea>
                </label>
                <label>
                    "Price (minor units):"
                    <input type="number" name="price" value=(offer.price.as_i32().to_string()) />
                </label>
                <label>
                    "Currency:"
                    <input type="text" name="currency" value=(offer.currency.as_str().to_string()) />
                </label>
                <button type="submit">"Save"</button>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct EditOfferForm {
    pub title: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub currency: Option<String>,
}

#[topcoat::router::page(POST)]
pub async fn update_offer(
    cx: &Cx,
    form: topcoat::router::content::Form<EditOfferForm>,
) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId(id_uuid);

    // Authorization check
    let _offer = crate::cx_helpers::owned_offer(cx, offer_id).await?;

    let (price, currency) =
        super::super::fields::parse_for_patch(form.0.price.as_deref(), form.0.currency.as_deref())?;

    let update_req = haven_domain::offer::UpdateOffer {
        title: form.0.title,
        description: form.0.description.filter(|s| !s.is_empty()),
        price,
        currency,
    };

    update_req
        .validate()
        .map_err(|_| bad_request("invalid offer data"))?;

    let user = crate::cx_helpers::require_auth(cx).await?;
    crate::cx_helpers::registry(cx)
        .offers()
        .update(offer_id, user.id, &update_req)
        .await
        .map_err(crate::cx_helpers::map_repo_err)?;

    Err(see_other(format!("/offers/{offer_id}")).into())
}
