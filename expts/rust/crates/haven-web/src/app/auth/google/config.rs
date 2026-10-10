//! Google OAuth 2.0 configuration types and environment variable parsing.

use url::Url;

/// Configuration for Google OAuth 2.0 authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoogleOAuthConfig {
    /// Google OAuth client ID.
    pub client_id: String,
    /// Google OAuth client secret.
    pub client_secret: String,
    /// Authorized OAuth redirect URI.
    pub redirect_uri: String,
}

/// Errors when validating or loading Google OAuth configuration.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OAuthConfigError {
    /// One credential was supplied without the other.
    #[error("Incomplete Google OAuth configuration: {0}")]
    Incomplete(String),
    /// Configured redirect URI could not be parsed as a valid URL.
    #[error("Invalid redirect URI: {0}")]
    InvalidRedirectUri(String),
}

/// Raw environment variables or inputs for Google OAuth configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RawGoogleOAuthConfig {
    /// Google OAuth client ID.
    pub client_id: Option<String>,
    /// Google OAuth client secret.
    pub client_secret: Option<String>,
    /// Explicit authorized OAuth redirect URI.
    pub redirect_uri: Option<String>,
    /// Public base URL used to construct redirect URI if not explicitly provided.
    pub public_base_url: Option<String>,
    /// Optional custom callback endpoint path (defaults to `/auth/google/callback`).
    pub callback_endpoint: Option<String>,
}

impl GoogleOAuthConfig {
    /// Pure parser for Google OAuth configuration.
    ///
    /// Accepts optionals for credentials, redirect URI, base URL, and callback endpoint to allow
    /// deterministic testing without mutating process environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`OAuthConfigError::Incomplete`] if only one credential is provided,
    /// or [`OAuthConfigError::InvalidRedirectUri`] if the constructed redirect URI is malformed.
    pub fn parse(raw: RawGoogleOAuthConfig) -> Result<Option<Self>, OAuthConfigError> {
        let client_id = raw
            .client_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let client_secret = raw
            .client_secret
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
                let redirect_uri = raw
                    .redirect_uri
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        let raw_base = raw
                            .public_base_url
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
                        let endpoint = raw
                            .callback_endpoint
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
    ///
    /// # Errors
    ///
    /// Returns [`url::ParseError`] if URL constructing fails.
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
    ///
    /// # Errors
    ///
    /// Returns [`OAuthConfigError`] if configuration environment variables are invalid.
    pub fn from_env() -> Result<Option<Self>, OAuthConfigError> {
        Self::parse(RawGoogleOAuthConfig {
            client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            redirect_uri: std::env::var("GOOGLE_REDIRECT_URI").ok(),
            public_base_url: std::env::var("PUBLIC_BASE_URL").ok(),
            callback_endpoint: std::env::var("GOOGLE_CALLBACK_ENDPOINT").ok(),
        })
    }
}
