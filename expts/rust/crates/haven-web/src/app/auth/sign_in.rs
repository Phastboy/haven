
use topcoat::{
    context::Cx,
    router::error::see_other,
    Result as TopcoatResult,
};

#[topcoat::router::page]
pub async fn sign_in_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let _ = cx;
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
    let ip = crate::cx_helpers::client_ip_key(cx);
    let limiter = crate::cx_helpers::sign_in_limiter(cx);
    
    if limiter.try_acquire(&ip).is_err() {
        return Err(see_other("/auth/sent").into());
    }
    
    let Ok(email) = haven_domain::account::Email::parse(&form.0.email) else {
        return Err(see_other("/auth/sent").into());
    };
    
    if limiter.try_acquire(email.as_str()).is_err() {
        return Err(see_other("/auth/sent").into());
    }
    
    let registry = crate::cx_helpers::registry(cx);
    let map_err = crate::cx_helpers::map_repo_err;
    
    // Find or create account
    let _ = match registry.accounts().find_by_email(&email).await.map_err(map_err)? {
        Some(acc) => acc,
        None => registry.accounts().create(&email).await.map_err(map_err)?,
    };
    
    let plaintext_token = haven_domain::session::PlaintextToken::generate().map_err(topcoat::Error::from)?;
    let hashed_token = plaintext_token.to_hashed();
    let expires_at = chrono::Utc::now().checked_add_signed(chrono::Duration::minutes(15)).ok_or_else(|| topcoat::Error::msg("Time overflow"))?;
    
    registry.magic_links().create(&email, &hashed_token, expires_at).await.map_err(map_err)?;
    
    let mut public_base_url = std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    if let Ok(mut parsed_url) = url::Url::parse(&public_base_url) {
        let host = parsed_url.host_str().unwrap_or("");
        if parsed_url.scheme() == "http" && host != "localhost" && host != "127.0.0.1" && host != "[::1]" {
            parsed_url.set_scheme("https").ok();
            public_base_url = parsed_url.to_string().trim_end_matches('/').to_string();
        }
    }
    
    let mail = topcoat::mail::mail! {
        from: "noreply@haven.localhost",
        to: email.as_str(),
        subject: "Sign in to Haven",
        text: format!("Click here to sign in: {}/auth/verify?token={}", public_base_url, plaintext_token.as_str())
    }?;
    
    topcoat::mail::send(cx, mail).await?;
    
    Err(see_other("/auth/sent").into())
}
