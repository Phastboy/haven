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
    form: topcoat::router::content::Form<SignInForm>,
) -> TopcoatResult<()> {
    let email_str = form.0.email;
    let ip = crate::cx_helpers::client_ip_key(cx);
    let limiter = crate::cx_helpers::sign_in_limiter(cx);
    
    let ip_ok = limiter.try_acquire(&ip).is_ok();
    let email_ok = limiter.try_acquire(&email_str).is_ok();
    
    if !ip_ok || !email_ok {
        return Err(redirect("/auth/sent").into());
    }
    
    let Ok(email) = haven_domain::account::Email::parse(&email_str) else {
        return Err(redirect("/auth/sent").into());
    };
    
    let db = crate::cx_helpers::db(cx);
    
    // Find or create account
    let account = match haven_db::accounts::find_by_email(db, &email).await.map_err(topcoat::Error::from)? {
        Some(acc) => acc,
        None => haven_db::accounts::create(db, &email).await.map_err(topcoat::Error::from)?,
    };
    
    let plaintext_token = haven_domain::session::PlaintextToken::generate().map_err(topcoat::Error::from)?;
    let hashed_token = plaintext_token.to_hashed();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);
    
    haven_db::magic_links::create(db, &email, &hashed_token, expires_at).await.map_err(topcoat::Error::from)?;
    
    let mail = topcoat::mail::mail! {
        from: "noreply@haven.localhost",
        to: email.as_str(),
        subject: "Sign in to Haven",
        text: format!("Click here to sign in: http://localhost:8080/auth/verify?token={}", plaintext_token.as_str())
    }.map_err(topcoat::Error::from)?;
    
    topcoat::mail::send(cx, mail).await?;
    
    Err(redirect("/auth/sent").into())
}
