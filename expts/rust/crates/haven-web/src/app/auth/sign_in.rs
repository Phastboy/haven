#![allow(unused_variables)]
use topcoat::{
    context::Cx,
    router::error::redirect,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn sign_in_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    Ok(topcoat::view::view! {
        <div class="sign-in">
            <h1>"Sign In"</h1>
            <form method="post" action="/auth/sign-in">
                <input type="email" name="email" required="true" placeholder="Enter your email" />
                <button type="submit">"Send Magic Link"</button>
            </form>
        </div>
    })
}

#[derive(serde::Deserialize)]
pub struct SignInForm {
    pub email: String,
}

#[topcoat::router::page(POST)]
pub async fn submit_sign_in(
    cx: &Cx,
    _form: topcoat::router::content::Form<SignInForm>,
) -> TopcoatResult<()> {
    let ip = crate::cx_helpers::client_ip_key(cx);
    let limiter = crate::cx_helpers::sign_in_limiter(cx);
    crate::cx_helpers::enforce(limiter, &ip)?;
    
    // Simulate DB interaction for now
    
    Err(redirect("/auth/sent").into())
}
