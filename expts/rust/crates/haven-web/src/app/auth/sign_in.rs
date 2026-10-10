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
    if crate::app::auth::guard::current_user(cx).await?.is_some() {
        return Err(topcoat::router::error::redirect("/offers").into());
    }

    let query = topcoat::router::parse_query_params::<SignInQuery>(cx).ok();
    let error_message = query.and_then(|q| q.error).map(|err| match err.as_str() {
        "google_cancelled" => "Google sign-in was cancelled.".to_string(),
        "state_mismatch" | "state_missing" => {
            "Authentication session expired. Please try again.".to_string()
        }
        "oauth_disabled" => "Google sign-in is currently unavailable.".to_string(),
        "email_delivery_unavailable" => {
            "Email sign-in is temporarily unavailable while mail delivery is being configured. Please sign in with Google.".to_string()
        }
        "token_exchange_failed" | "userinfo_failed" => {
            "Unable to verify Google account. Please try again or use email sign-in.".to_string()
        }
        "email_unverified" => "Your Google email address is unverified.".to_string(),
        _ => "An error occurred while signing in. Please try again.".to_string(),
    });

    let google_enabled = crate::app::state::google_oauth(cx).is_some();
    let email_delivery_enabled = crate::app::state::email_delivery_enabled(cx);

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
                        variant: if email_delivery_enabled {
                            ButtonVariant::Secondary
                        } else {
                            ButtonVariant::Primary
                        },
                    )
                </div>
                <div class="auth-divider">
                    <span>"or"</span>
                </div>
            }
            if !email_delivery_enabled {
                <div class="auth-notice" role="status">
                    "Email sign-in is temporarily unavailable while mail delivery infrastructure is being configured. Please sign in with Google above."
                </div>
                <form method="post" action="/auth/sign-in" class="form-stack">
                    text_field(
                        label: "Email",
                        name: "email",
                        field_type: "email",
                        required: false,
                        disabled: true,
                        placeholder: Some("name@example.com".to_string()),
                        help: Some("Direct email sign-in will be enabled soon.".to_string()),
                    )
                    button(
                        text: "Send Magic Link (Unavailable)",
                        variant: ButtonVariant::Secondary,
                        disabled: true,
                    )
                </form>
            } else {
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
            }
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
    if !crate::app::state::email_delivery_enabled(cx) {
        return Err(see_other("/auth/sign-in?error=email_delivery_unavailable").into());
    }

    let ip = crate::rate_limit::client_ip_key(cx);
    let limiter = crate::rate_limit::sign_in_limiter(cx);

    if limiter.try_acquire(&ip).is_err() {
        return Err(see_other("/auth/sent").into());
    }

    let Ok(email) = haven_domain::account::Email::parse(&form.0.email) else {
        return Err(see_other("/auth/sent").into());
    };

    if limiter.try_acquire(email.as_str()).is_err() {
        return Err(see_other("/auth/sent").into());
    }

    let registry = crate::app::state::registry(cx);
    let map_err = crate::app::state::map_repo_err;

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

    let raw_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let public_base_url = resolve_public_base_url(&raw_base_url);

    let mail = topcoat::mail::mail! {
        from: "noreply@haven.localhost",
        to: email.as_str(),
        subject: "Sign in to Haven",
        text: format!("Click here to sign in: {}/auth/verify?token={}", public_base_url, plaintext_token.as_str())
    }?;

    topcoat::mail::send(cx, mail).await?;

    Err(see_other("/auth/sent").into())
}

/// Resolves and normalizes the public base URL used for sign-in links.
///
/// Parses the configured URL to preserve valid schemes (case-insensitively, e.g. `HTTPS://`),
/// prepending `http://` only when the input lacks an HTTP or HTTPS scheme.
/// Non-loopback `http` hosts are upgraded to `https`.
/// Trailing slashes are stripped to avoid duplicate slashes when appending route paths.
#[must_use]
pub fn resolve_public_base_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut parsed_url = match url::Url::parse(trimmed) {
        Ok(parsed) if parsed.scheme() == "http" || parsed.scheme() == "https" => parsed,
        _ => match url::Url::parse(&format!("http://{trimmed}")) {
            Ok(parsed) => parsed,
            Err(_) => return "http://localhost:8080".to_string(),
        },
    };

    let host = parsed_url.host_str().unwrap_or("");
    if parsed_url.scheme() == "http"
        && host != "localhost"
        && host != "127.0.0.1"
        && host != "[::1]"
    {
        parsed_url.set_scheme("https").ok();
    }

    parsed_url.to_string().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_public_base_url_preserves_uppercase_and_valid_schemes() {
        assert_eq!(
            resolve_public_base_url("HTTPS://example.com"),
            "https://example.com"
        );
        assert_eq!(
            resolve_public_base_url("HTTP://localhost:8080"),
            "http://localhost:8080"
        );
        assert_eq!(
            resolve_public_base_url("https://example.com/"),
            "https://example.com"
        );
    }

    #[test]
    fn test_resolve_public_base_url_prepends_scheme_when_missing() {
        assert_eq!(
            resolve_public_base_url("localhost:8080"),
            "http://localhost:8080"
        );
        assert_eq!(
            resolve_public_base_url("127.0.0.1:8080"),
            "http://127.0.0.1:8080"
        );
        assert_eq!(resolve_public_base_url("[::1]:8080"), "http://[::1]:8080");
        // Non-loopback schemeless host is prepended and upgraded to https
        assert_eq!(
            resolve_public_base_url("example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn test_resolve_public_base_url_fallback_for_empty() {
        assert_eq!(resolve_public_base_url(""), "http://localhost:8080");
        assert_eq!(resolve_public_base_url("   "), "http://localhost:8080");
    }
}
