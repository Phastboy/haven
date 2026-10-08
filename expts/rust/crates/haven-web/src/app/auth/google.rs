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
}
