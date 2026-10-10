//! Google OAuth 2.0 configuration and state management.

use url::Url;

/// Configuration for Google OAuth 2.0 authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Errors when validating or loading Google OAuth configuration.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OAuthConfigError {
    #[error("Incomplete Google OAuth configuration: {0}")]
    Incomplete(String),
    #[error("Invalid redirect URI: {0}")]
    InvalidRedirectUri(String),
}

impl GoogleOAuthConfig {
    /// Pure parser for Google OAuth configuration.
    ///
    /// Accepts optionals for credentials, redirect URI, and base URL to allow
    /// deterministic testing without mutating process environment variables.
    #[cfg(test)]
    pub fn parse(
        client_id: Option<String>,
        client_secret: Option<String>,
        redirect_uri: Option<String>,
        public_base_url: Option<String>,
    ) -> Result<Option<Self>, OAuthConfigError> {
        Self::parse_with_endpoint(
            client_id,
            client_secret,
            redirect_uri,
            public_base_url,
            None,
        )
    }

    /// Pure parser for Google OAuth configuration supporting explicit callback endpoints.
    pub fn parse_with_endpoint(
        client_id: Option<String>,
        client_secret: Option<String>,
        redirect_uri: Option<String>,
        public_base_url: Option<String>,
        callback_endpoint: Option<String>,
    ) -> Result<Option<Self>, OAuthConfigError> {
        let client_id = client_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let client_secret = client_secret
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        match (client_id, client_secret) {
            (None, None) => Ok(None),
            (Some(_), None) => Err(OAuthConfigError::Incomplete(
                "GOOGLE_CLIENT_ID is set but GOOGLE_CLIENT_SECRET is missing".into(),
            )),
            (None, Some(_)) => Err(OAuthConfigError::Incomplete(
                "GOOGLE_CLIENT_SECRET is set but GOOGLE_CLIENT_ID is missing".into(),
            )),
            (Some(client_id), Some(client_secret)) => {
                let redirect_uri = redirect_uri
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        let raw_base = public_base_url
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "http://localhost:3000".to_string());
                        let base = if raw_base.starts_with("http://")
                            || raw_base.starts_with("https://")
                        {
                            raw_base
                        } else {
                            format!("http://{raw_base}")
                        };
                        let endpoint = callback_endpoint
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "/auth/google/callback".to_string());
                        let endpoint = if endpoint.starts_with('/') {
                            endpoint
                        } else {
                            format!("/{endpoint}")
                        };
                        format!("{}{}", base.trim_end_matches('/'), endpoint)
                    });

                Url::parse(&redirect_uri)
                    .map_err(|e| OAuthConfigError::InvalidRedirectUri(e.to_string()))?;

                Ok(Some(Self {
                    client_id,
                    client_secret,
                    redirect_uri,
                }))
            }
        }
    }

    /// Builds the Google OAuth 2.0 authorization consent URL with CSRF state.
    pub fn authorization_url(&self, state: &str) -> Result<String, url::ParseError> {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", state)
            .append_pair("access_type", "online")
            .append_pair("prompt", "select_account");
        Ok(url.to_string())
    }

    /// Loads Google OAuth configuration from environment variables.
    pub fn from_env() -> Result<Option<Self>, OAuthConfigError> {
        Self::parse_with_endpoint(
            std::env::var("GOOGLE_CLIENT_ID").ok(),
            std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            std::env::var("GOOGLE_REDIRECT_URI").ok(),
            std::env::var("PUBLIC_BASE_URL").ok(),
            std::env::var("GOOGLE_CALLBACK_ENDPOINT").ok(),
        )
    }
}

/// Cookie name used to persist the OAuth CSRF state across the consent redirect.
pub const OAUTH_STATE_COOKIE_NAME: &str = "g_oauth_state";

/// GET /auth/google
///
/// Initiates the Google OAuth 2.0 flow:
/// 1. Redirects authenticated users directly to /offers.
/// 2. If Google OAuth is unconfigured, falls back to /auth/sign-in.
/// 3. Generates a cryptographically random CSRF state token stored in a short-lived cookie.
/// 4. Redirects (303 See Other) to Google's consent screen.
#[topcoat::router::page]
pub async fn google_sign_in(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    use topcoat::{
        cookie::{Cookie, Cookies, SameSite, time::Duration},
        router::error::see_other,
    };

    if crate::cx_helpers::current_user(cx).await?.is_some() {
        return Err(see_other("/offers").into());
    }

    let Some(config) = crate::cx_helpers::google_oauth(cx) else {
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

/// Query parameters for GET /auth/google/callback.
#[derive(serde::Deserialize, Default, Debug)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Token response returned by Google's OAuth 2.0 token endpoint.
#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub id_token: Option<String>,
}

/// Userinfo response returned by Google's userinfo endpoint.
#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GoogleUserInfo {
    pub email: String,
    pub email_verified: Option<bool>,
}

/// Exchanges an authorization code for Google OAuth tokens.
pub async fn exchange_code_for_token(
    client: &reqwest::Client,
    config: &GoogleOAuthConfig,
    code: &str,
) -> Result<GoogleTokenResponse, String> {
    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        return Err(format!("Token exchange failed with HTTP {status}"));
    }

    res.json::<GoogleTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
}

/// Fetches the user profile information from Google's userinfo endpoint.
pub async fn fetch_user_info(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<GoogleUserInfo, String> {
    let res = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Userinfo request failed: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!(
            "Userinfo request failed with HTTP {status}: {body}"
        ));
    }

    res.json::<GoogleUserInfo>()
        .await
        .map_err(|e| format!("Failed to parse userinfo response: {e}"))
}

/// GET /auth/google/callback
///
/// Google OAuth 2.0 authorization callback:
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

    if crate::cx_helpers::current_user(cx).await?.is_some() {
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
    let Some(config) = crate::cx_helpers::google_oauth(cx) else {
        return Err(see_other("/auth/sign-in?error=oauth_disabled").into());
    };
    let client = crate::cx_helpers::http_client(cx);

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
    let registry = crate::cx_helpers::registry(cx);
    let map_err = crate::cx_helpers::map_repo_err;

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
    let hash_hex = crate::cx_helpers::token_hash_hex(&session.token_hash);
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
#[topcoat::router::page("./callback")]
pub async fn google_callback(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    handle_google_callback(cx).await
}

/// GET /auth/google/redirect
///
/// Alias route for /auth/google/callback to accommodate credentials configured
/// with /auth/google/redirect as their authorized redirect URI.
#[topcoat::router::page("./redirect")]
pub async fn google_redirect(cx: &topcoat::context::Cx) -> topcoat::Result<()> {
    handle_google_callback(cx).await
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    reason = "test assertions and mocks"
)]
mod tests;
