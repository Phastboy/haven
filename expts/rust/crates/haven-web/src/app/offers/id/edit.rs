
use topcoat::{
    context::Cx,
    router::error::see_other,
    view::view,
    Result as TopcoatResult,
};

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
                    <input type="number" name="price" value=(offer.price.map(|p| p.as_i32().to_string()).unwrap_or_default()) />
                </label>
                <label>
                    "Currency:"
                    <input type="text" name="currency" value=(offer.currency.map(|c| c.as_str().to_string()).unwrap_or_default()) />
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
    pub price: Option<i32>,
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
    
    let price = form.0.price.map(haven_domain::offer::Price::new).transpose().map_err(|_| bad_request("invalid price"))?;
    let currency = form.0.currency.filter(|c| !c.is_empty()).map(|c| haven_domain::offer::CurrencyCode::parse(&c)).transpose().map_err(|_| bad_request("invalid currency"))?;
    
    let update_req = haven_domain::offer::UpdateOffer {
        title: form.0.title,
        description: form.0.description.filter(|s| !s.is_empty()),
        price,
        currency,
    };
    
    update_req.validate().map_err(|_| bad_request("invalid offer data"))?;
    
    let db = crate::cx_helpers::db(cx);
    haven_db::offers::update(db, offer_id, &update_req).await.map_err(topcoat::Error::from)?;
    
    Err(see_other(format!("/offers/{}", offer_id)).into())
}
