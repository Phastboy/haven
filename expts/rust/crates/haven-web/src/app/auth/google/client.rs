//! Outbound HTTP client functions for Google OAuth token and userinfo endpoints.

use super::config::GoogleOAuthConfig;

/// Token response returned by Google's OAuth 2.0 token endpoint.
#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GoogleTokenResponse {
    /// OAuth access token.
    pub access_token: String,
    /// Token type (typically "Bearer").
    pub token_type: Option<String>,
    /// Token lifetime in seconds.
    pub expires_in: Option<u64>,
    /// `OpenID` Connect identity token.
    pub id_token: Option<String>,
}

/// Userinfo response returned by Google's userinfo endpoint.
#[derive(serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GoogleUserInfo {
    /// User's email address.
    pub email: String,
    /// Whether the email address has been verified by Google.
    pub email_verified: Option<bool>,
}

/// Exchanges an authorization code for Google OAuth tokens.
///
/// # Errors
///
/// Returns an error message string if the HTTP request fails or response parsing fails.
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
///
/// # Errors
///
/// Returns an error message string if the HTTP request fails or response parsing fails.
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
