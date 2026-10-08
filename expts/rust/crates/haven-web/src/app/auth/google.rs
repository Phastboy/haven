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
    pub fn parse(
        client_id: Option<String>,
        client_secret: Option<String>,
        redirect_uri: Option<String>,
        public_base_url: Option<String>,
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
                        let base = public_base_url
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "http://localhost:3000".to_string());
                        format!("{}/auth/google/callback", base.trim_end_matches('/'))
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
        Self::parse(
            std::env::var("GOOGLE_CLIENT_ID").ok(),
            std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            std::env::var("GOOGLE_REDIRECT_URI").ok(),
            std::env::var("PUBLIC_BASE_URL").ok(),
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn parse_returns_none_when_both_credentials_are_absent() {
        let config = GoogleOAuthConfig::parse(None, None, None, None).unwrap_or(None);
        assert_eq!(config, None);
    }

    #[test]
    fn parse_returns_none_when_credentials_are_empty_strings() {
        let config = GoogleOAuthConfig::parse(
            Some("  ".into()),
            Some(String::new()),
            None,
            None,
        )
        .unwrap_or(None);
        assert_eq!(config, None);
    }

    #[test]
    fn parse_rejects_missing_secret() {
        let res = GoogleOAuthConfig::parse(
            Some("my-client-id".into()),
            None,
            None,
            None,
        );
        assert!(matches!(res, Err(OAuthConfigError::Incomplete(_))));
    }

    #[test]
    fn parse_rejects_missing_client_id() {
        let res = GoogleOAuthConfig::parse(
            None,
            Some("my-client-secret".into()),
            None,
            None,
        );
        assert!(matches!(res, Err(OAuthConfigError::Incomplete(_))));
    }

    #[test]
    fn parse_succeeds_with_explicit_redirect_uri() {
        let config = GoogleOAuthConfig::parse(
            Some("client-id-123".into()),
            Some("client-secret-456".into()),
            Some("https://example.com/custom/callback".into()),
            None,
        )
        .unwrap_or(None)
        .unwrap_or_else(|| panic!("expected Some(config)"));

        assert_eq!(config.client_id, "client-id-123");
        assert_eq!(config.client_secret, "client-secret-456");
        assert_eq!(config.redirect_uri, "https://example.com/custom/callback");
    }

    #[test]
    fn parse_defaults_redirect_uri_from_public_base_url() {
        let config = GoogleOAuthConfig::parse(
            Some("client-id-123".into()),
            Some("client-secret-456".into()),
            None,
            Some("https://haven.app/".into()),
        )
        .unwrap_or(None)
        .unwrap_or_else(|| panic!("expected Some(config)"));

        assert_eq!(config.redirect_uri, "https://haven.app/auth/google/callback");
    }

    #[test]
    fn parse_rejects_invalid_redirect_uri() {
        let res = GoogleOAuthConfig::parse(
            Some("client-id-123".into()),
            Some("client-secret-456".into()),
            Some("not-a-valid-url".into()),
            None,
        );
        assert!(matches!(res, Err(OAuthConfigError::InvalidRedirectUri(_))));
    }

    #[test]
    fn authorization_url_generates_valid_google_consent_url() {
        let config = GoogleOAuthConfig {
            client_id: "test-client-id.apps.googleusercontent.com".into(),
            client_secret: "test-secret".into(),
            redirect_uri: "http://localhost:3000/auth/google/callback".into(),
        };

        let raw_url = config.authorization_url("secure-csrf-token").unwrap();
        let parsed = Url::parse(&raw_url).unwrap();

        assert_eq!(parsed.scheme(), "https");
        assert_eq!(parsed.host_str(), Some("accounts.google.com"));
        assert_eq!(parsed.path(), "/o/oauth2/v2/auth");

        let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();
        assert_eq!(
            pairs.get("client_id").map(String::as_str),
            Some("test-client-id.apps.googleusercontent.com")
        );
        assert_eq!(
            pairs.get("redirect_uri").map(String::as_str),
            Some("http://localhost:3000/auth/google/callback")
        );
        assert_eq!(pairs.get("response_type").map(String::as_str), Some("code"));
        assert_eq!(
            pairs.get("scope").map(String::as_str),
            Some("openid email profile")
        );
        assert_eq!(
            pairs.get("state").map(String::as_str),
            Some("secure-csrf-token")
        );
        assert_eq!(pairs.get("access_type").map(String::as_str), Some("online"));
        assert_eq!(pairs.get("prompt").map(String::as_str), Some("select_account"));
    }

    #[test]
    fn cookie_constant_matches_expected_name() {
        assert_eq!(OAUTH_STATE_COOKIE_NAME, "g_oauth_state");
    }
}
