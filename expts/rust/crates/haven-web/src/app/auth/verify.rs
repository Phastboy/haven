#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    Result as TopcoatResult,
};

#[derive(serde::Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

#[topcoat::router::page]
pub async fn verify_prompt(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let query = topcoat::router::parse_query_params::<VerifyQuery>(cx)?;
    let token = query.token;
    Ok(topcoat::view::view! {
        <div class="verify">
            <h1>"Sign In Verification"</h1>
            <form method="post" action="/auth/verify">
                <input type="hidden" name="token" value=(token) />
                <button type="submit">"Click here to sign in"</button>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct VerifyForm {
    pub token: String,
}

#[topcoat::router::page(POST)]
pub async fn verify_submit(
    cx: &Cx,
    _form: topcoat::router::content::Form<VerifyForm>,
) -> TopcoatResult<()> {
    // 1. Parse token, hash it, find in DB, check expiry
    // 2. Create session in DB, set cookie
    // Redirect to /offers
    Err(redirect("/offers").into())
}
