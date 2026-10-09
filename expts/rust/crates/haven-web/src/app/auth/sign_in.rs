use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::field::text_field;
use crate::app::components::form_error::form_error;
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

#[derive(serde::Deserialize, Default)]
pub struct SignInQuery {
    pub error: Option<String>,
}

#[topcoat::router::page]
pub async fn sign_in_page(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    if crate::cx_helpers::current_user(cx).await?.is_some() {
        return Err(topcoat::router::error::redirect("/offers").into());
    }

    let query = topcoat::router::parse_query_params::<SignInQuery>(cx).ok();
    let error_message = query.and_then(|q| q.error).map(|err| match err.as_str() {
        "google_cancelled" => "Google sign-in was cancelled.".to_string(),
        "state_mismatch" | "state_missing" => {
            "Authentication session expired. Please try again.".to_string()
        }
        "oauth_disabled" => "Google sign-in is currently unavailable.".to_string(),
        "token_exchange_failed" | "userinfo_failed" => {
            "Unable to verify Google account. Please try again or use email sign-in.".to_string()
        }
        "email_unverified" => "Your Google email address is unverified.".to_string(),
        _ => "An error occurred while signing in. Please try again.".to_string(),
    });

    let google_enabled = crate::cx_helpers::google_oauth(cx).is_some();

    Ok(view! {
        <div class="auth-card">
            <h1>"Sign In"</h1>
            if let Some(msg) = error_message {
                form_error(message: msg)
            }
            if google_enabled {
                <div class="google-auth">
                    button_link(
                        href: "/auth/google",
                        text: "Sign in with Google",
                        variant: ButtonVariant::Secondary,
                    )
                </div>
                <div class="auth-divider">
                    <span>"or"</span>
                </div>
            }
            <form method="post" action="/auth/sign-in" class="form-stack">
                text_field(
                    label: "Email",
                    name: "email",
                    field_type: "email",
                    required: true,
                    placeholder: Some("name@example.com".to_string()),
                )
                button(
                    text: "Send Magic Link",
                    variant: ButtonVariant::Primary,
                )
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
    let _ = match registry
        .accounts()
        .find_by_email(&email)
        .await
        .map_err(map_err)?
    {
        Some(acc) => acc,
        None => registry.accounts().create(&email).await.map_err(map_err)?,
    };

    let plaintext_token =
        haven_domain::session::PlaintextToken::generate().map_err(topcoat::Error::from)?;
    let hashed_token = plaintext_token.to_hashed();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .ok_or_else(|| topcoat::Error::msg("Time overflow"))?;

    registry
        .magic_links()
        .create(&email, &hashed_token, expires_at)
        .await
        .map_err(map_err)?;

    let mut public_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    if let Ok(mut parsed_url) = url::Url::parse(&public_base_url) {
        let host = parsed_url.host_str().unwrap_or("");
        if parsed_url.scheme() == "http"
            && host != "localhost"
            && host != "127.0.0.1"
            && host != "[::1]"
        {
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
