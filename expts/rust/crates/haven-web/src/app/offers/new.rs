#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    view::view,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn new_offer_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    Ok(view! {
        <div class="new-offer">
            <h1>"New Offer"</h1>
            <form method="post" action="/offers/new">
                <input type="hidden" name="idempotency_key" value=(idempotency_key) />
                <label>
                    "Title:"
                    <input type="text" name="title" required="required" />
                </label>
                <label>
                    "Description:"
                    <textarea name="description"></textarea>
                </label>
                <label>
                    "Price (minor units):"
                    <input type="number" name="price" />
                </label>
                <label>
                    "Currency:"
                    <input type="text" name="currency" />
                </label>
                <button type="submit">"Create"</button>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct NewOfferForm {
    pub idempotency_key: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub currency: Option<String>,
}

#[topcoat::router::page(POST)]
pub async fn create_offer(
    cx: &Cx,
    form: topcoat::router::content::Form<NewOfferForm>,
) -> TopcoatResult<()> {
    let user = crate::cx_helpers::require_auth(cx).await?;
    
    let limiter = crate::cx_helpers::create_offer_limiter(cx);
    crate::cx_helpers::enforce(limiter, &user.id.to_string())?;

    let price = form.0.price.map(haven_domain::offer::Price::new).transpose().map_err(|_| topcoat::router::error::bad_request("invalid price"))?;
    let currency = form.0.currency.filter(|c| !c.is_empty()).map(|c| haven_domain::offer::CurrencyCode::parse(&c)).transpose().map_err(|_| topcoat::router::error::bad_request("invalid currency"))?;
    
    let create_req = haven_domain::offer::CreateOffer {
        title: form.0.title,
        description: form.0.description.filter(|s| !s.is_empty()),
        price,
        currency,
    };
    
    create_req.validate().map_err(|_| topcoat::router::error::bad_request("invalid offer data"))?;

    let db = crate::cx_helpers::db(cx);
    let offer = haven_db::offers::create(db, user.id, &create_req, form.0.idempotency_key).await.map_err(topcoat::Error::from)?;
    
    Err(redirect(format!("/offers/{}", offer.id)).into())
}
