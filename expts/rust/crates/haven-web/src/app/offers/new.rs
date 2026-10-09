use topcoat::{
    Result as TopcoatResult,
    router::error::{redirect_permanent, see_other},
    view::view,
};

#[topcoat::router::page]
pub async fn new_offer_redirect() -> TopcoatResult<impl topcoat::view::View> {
    if false {
        return Ok(view! {});
    }
    Err(see_other("/offers/manage/new").into())
}

#[topcoat::router::page(POST)]
pub async fn create_offer_redirect() -> TopcoatResult<()> {
    Err(redirect_permanent("/offers/manage/new").into())
}
