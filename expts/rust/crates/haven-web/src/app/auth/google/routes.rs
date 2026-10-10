//! Route handlers and state management for Google OAuth 2.0 flow.

use super::client::{exchange_code_for_token, fetch_user_info};

/// Cookie name used to persist the OAuth CSRF state across the consent redirect.
pub const OAUTH_STATE_COOKIE_NAME: &str = "g_oauth_state";

/// Query parameters for GET /auth/google/callback.
#[derive(serde::Deserialize, Default, Debug)]
pub struct CallbackQuery {
    /// Authorization code returned by Google.
    pub code: Option<String>,
    /// CSRF state token returned by Google.
    pub state: Option<String>,
    /// Error parameter returned if consent was denied.
    pub error: Option<String>,
}

/// GET /auth/google
///
/// Initiates the Google OAuth 2.0 flow:
/// 1. Redirects authenticated users directly to /offers.
/// 2. If Google OAuth is unconfigured, falls back to /auth/sign-in.
/// 3. Generates a cryptographically random CSRF state token stored in a short-lived cookie.
/// 4. Redirects (303 See Other) to Google's consent screen.
#[topcoat::router::page("/auth/google")]
pub async fn google_sign_in(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    use topcoat::{
        cookie::{Cookie, Cookies, SameSite, time::Duration},
        router::error::see_other,
    };

    if crate::app::auth::guard::current_user(cx).await?.is_some() {
        return Err(see_other("/offers").into());
    }

    let Some(config) = crate::app::state::google_oauth(cx) else {
        return Err(see_other("/auth/sign-in").into());
    };

    let state = haven_domain::session::PlaintextToken::generate()
        .map_err(|e| topcoat::Error::msg(e.to_string()))?;

    let mut cookie = Cookie::new(OAUTH_STATE_COOKIE_NAME, state.as_str().to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/auth/google");
    cookie.set_max_age(Duration::seconds(600));
    if config.redirect_uri.starts_with("https://") {
        cookie.set_secure(true);
    }

    topcoat::cookie::cookies(cx).add(cookie);

    let auth_url = config
        .authorization_url(state.as_str())
        .map_err(|e| topcoat::Error::msg(e.to_string()))?;

    Err(see_other(auth_url).into())
}

/// Google OAuth 2.0 authorization callback logic:
/// 1. Verifies the query parameters (`code` and `state`).
/// 2. Validates CSRF `state` matches the `g_oauth_state` cookie and removes the cookie.
/// 3. Exchanges authorization code for Google access token.
/// 4. Retrieves Google user info and ensures email is verified.
/// 5. Finds or creates the Account and User in PostgreSQL.
/// 6. Issues a session cookie and redirects to `/offers/new`.
async fn handle_google_callback(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    use topcoat::{
        cookie::{Cookie, Cookies},
        router::error::see_other,
    };

    if crate::app::auth::guard::current_user(cx).await?.is_some() {
        return Err(see_other("/offers").into());
    }

    let query = topcoat::router::parse_query_params::<CallbackQuery>(cx).unwrap_or_default();

    if query.error.is_some() {
        return Err(see_other("/auth/sign-in?error=google_cancelled").into());
    }

    let (Some(code), Some(incoming_state)) = (query.code, query.state) else {
        return Err(see_other("/auth/sign-in?error=invalid_request").into());
    };

    // 1. Verify and clear CSRF state cookie
    let cookie_jar = topcoat::cookie::cookies(cx);
    let stored_state = cookie_jar
        .get(OAUTH_STATE_COOKIE_NAME)
        .map(|c| c.value().to_string());

    // Always clear the state cookie immediately to prevent replay attacks
    let mut removal = Cookie::new(OAUTH_STATE_COOKIE_NAME, "");
    removal.set_path("/auth/google");
    cookie_jar.remove(removal);

    let Some(stored_state) = stored_state else {
        return Err(see_other("/auth/sign-in?error=state_missing").into());
    };

    if stored_state != incoming_state {
        return Err(see_other("/auth/sign-in?error=state_mismatch").into());
    }

    // 2. Load Google OAuth config and HTTP client
    let Some(config) = crate::app::state::google_oauth(cx) else {
        return Err(see_other("/auth/sign-in?error=oauth_disabled").into());
    };
    let client = crate::app::state::http_client(cx);

    // 3. Exchange code for access token
    let Ok(tokens) = exchange_code_for_token(client, config, &code).await else {
        return Err(see_other("/auth/sign-in?error=token_exchange_failed").into());
    };

    // 4. Retrieve user info from Google
    let Ok(user_info) = fetch_user_info(client, &tokens.access_token).await else {
        return Err(see_other("/auth/sign-in?error=userinfo_failed").into());
    };

    // Security: Only accept verified email addresses from Google
    if user_info.email_verified != Some(true) {
        return Err(see_other("/auth/sign-in?error=email_unverified").into());
    }

    let Ok(email) = haven_domain::account::Email::parse(&user_info.email) else {
        return Err(see_other("/auth/sign-in?error=invalid_email").into());
    };

    // 5. Account and User management
    let registry = crate::app::state::registry(cx);
    let map_err = crate::app::state::map_repo_err;

    let account = if let Some(acc) = registry
        .accounts()
        .find_by_email(&email)
        .await
        .map_err(map_err)?
    {
        if !acc.email_verified {
            registry
                .accounts()
                .mark_verified(acc.id)
                .await
                .map_err(map_err)?;
        }
        acc
    } else {
        let acc = registry.accounts().create(&email).await.map_err(map_err)?;
        registry
            .accounts()
            .mark_verified(acc.id)
            .await
            .map_err(map_err)?;
        acc
    };

    // Ensure User record exists (created on first sign-in)
    let user = registry
        .users()
        .find_by_account(account.id)
        .await
        .map_err(map_err)?;
    if user.is_none() {
        registry.users().create(account.id).await.map_err(map_err)?;
    }

    // 6. Issue session
    let session = topcoat::session::start(cx).await?;
    let hash_hex = crate::app::auth::guard::token_hash_hex(&session.token_hash);
    let hashed_token = haven_domain::session::HashedToken::from_hex(hash_hex);
    let expires_at = chrono::DateTime::<chrono::Utc>::from(session.expires_at);

    let ip = topcoat::router::request::client_ip(cx).map(|ip| ip.to_string());
    let user_agent = topcoat::router::request::headers(cx)
        .get("user-agent")
        .and_then(|h| h.to_str().ok().map(std::string::ToString::to_string));

    let ip_addr = ip.and_then(|s| s.parse::<std::net::IpAddr>().ok());
    registry
        .sessions()
        .create(
            account.id,
            &hashed_token,
            expires_at,
            ip_addr,
            user_agent.as_deref(),
        )
        .await
        .map_err(map_err)?;

    Err(see_other("/offers/new").into())
}

/// GET /auth/google/callback
///
/// Handles the Google OAuth 2.0 authorization callback.
#[topcoat::router::page("/auth/google/callback")]
pub async fn google_callback(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    handle_google_callback(cx).await
}

/// GET /auth/google/redirect
///
/// Alias route for /auth/google/callback to accommodate credentials configured
/// with /auth/google/redirect as their authorized redirect URI.
#[topcoat::router::page("/auth/google/redirect")]
pub async fn google_redirect(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    handle_google_callback(cx).await
}
