use super::*;

#[test]
fn parse_returns_none_when_both_credentials_are_absent() {
    let config = GoogleOAuthConfig::parse(None, None, None, None).unwrap_or(None);
    assert_eq!(config, None);
}

#[test]
fn parse_returns_none_when_credentials_are_empty_strings() {
    let config = GoogleOAuthConfig::parse(Some("  ".into()), Some(String::new()), None, None)
        .unwrap_or(None);
    assert_eq!(config, None);
}

#[test]
fn parse_rejects_missing_secret() {
    let res = GoogleOAuthConfig::parse(Some("my-client-id".into()), None, None, None);
    assert!(matches!(res, Err(OAuthConfigError::Incomplete(_))));
}

#[test]
fn parse_rejects_missing_client_id() {
    let res = GoogleOAuthConfig::parse(None, Some("my-client-secret".into()), None, None);
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

    assert_eq!(
        config.redirect_uri,
        "https://haven.app/auth/google/callback"
    );
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
    assert_eq!(
        pairs.get("prompt").map(String::as_str),
        Some("select_account")
    );
}

#[test]
fn cookie_constant_matches_expected_name() {
    assert_eq!(OAUTH_STATE_COOKIE_NAME, "g_oauth_state");
}

#[test]
fn deserializes_google_token_response() {
    let json = r#"{
        "access_token": "ya29.sample_access_token",
        "token_type": "Bearer",
        "expires_in": 3599,
        "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.sample"
    }"#;

    let tokens: GoogleTokenResponse = serde_json::from_str(json).unwrap();
    assert_eq!(tokens.access_token, "ya29.sample_access_token");
    assert_eq!(tokens.token_type.as_deref(), Some("Bearer"));
    assert_eq!(tokens.expires_in, Some(3599));
    assert!(tokens.id_token.is_some());
}

#[test]
fn deserializes_google_user_info_with_verified_email() {
    let json = r#"{
        "email": "developer@example.com",
        "email_verified": true
    }"#;

    let info: GoogleUserInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.email, "developer@example.com");
    assert_eq!(info.email_verified, Some(true));
}

#[test]
fn deserializes_google_user_info_with_unverified_email() {
    let json = r#"{
        "email": "unverified@example.com",
        "email_verified": false
    }"#;

    let info: GoogleUserInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.email, "unverified@example.com");
    assert_eq!(info.email_verified, Some(false));
}

#[test]
fn deserializes_callback_query_parameters() {
    let json = r#"{"code": "auth-code-123", "state": "random-state-456"}"#;
    let query: CallbackQuery = serde_json::from_str(json).unwrap();
    assert_eq!(query.code.as_deref(), Some("auth-code-123"));
    assert_eq!(query.state.as_deref(), Some("random-state-456"));
    assert!(query.error.is_none());
}

fn test_router(oauth_config: Option<GoogleOAuthConfig>) -> topcoat::router::Router {
    use topcoat::{
        cookie::RouterBuilderCookieExt, router::RouterBuilderDiscoverExt,
        session::RouterBuilderSessionExt,
    };

    struct DummyRegistry;
    impl haven_domain::ports::Registry for DummyRegistry {
        fn offers(&self) -> &(dyn haven_domain::ports::OfferRepository + 'static) {
            unimplemented!()
        }
        fn accounts(&self) -> &(dyn haven_domain::ports::AccountRepository + 'static) {
            unimplemented!()
        }
        fn users(&self) -> &(dyn haven_domain::ports::UserRepository + 'static) {
            unimplemented!()
        }
        fn magic_links(&self) -> &(dyn haven_domain::ports::MagicLinkRepository + 'static) {
            unimplemented!()
        }
        fn sessions(&self) -> &(dyn haven_domain::ports::SessionRepository + 'static) {
            unimplemented!()
        }
    }

    let state = crate::cx_helpers::AppState {
        registry: std::sync::Arc::new(DummyRegistry),
        google_oauth: oauth_config,
        http_client: reqwest::Client::new(),
    };

    crate::app::router()
        .cookies()
        .sessions(
            topcoat::session::SessionConfig::builder()
                .token_store(crate::token_store::AdaptiveCookieTokenStore::new().name("sid"))
                .build(),
        )
        .app_context(state)
        .app_context(crate::cx_helpers::SignInLimiter::new())
        .app_context(crate::cx_helpers::CreateOfferLimiter::new())
        .discover()
        .build()
}

#[tokio::test]
async fn test_google_redirect_when_oauth_disabled() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in"
    );
}

#[tokio::test]
async fn test_google_redirect_when_oauth_enabled() {
    let config = GoogleOAuthConfig {
        client_id: "test-client-id.apps.googleusercontent.com".into(),
        client_secret: "test-secret".into(),
        redirect_uri: "http://localhost:3000/auth/google/callback".into(),
    };
    let router = test_router(Some(config));
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);

    let location = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
    assert!(location.contains("client_id=test-client-id.apps.googleusercontent.com"));
    assert!(
        location.contains("redirect_uri=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fgoogle%2Fcallback")
    );
    assert!(location.contains("state="));

    // Ensure CSRF state cookie is set
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("g_oauth_state="));
    assert!(set_cookie.contains("Path=/auth/google"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));
}

#[tokio::test]
async fn test_callback_redirects_when_user_cancels() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google/callback?error=access_denied")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=google_cancelled"
    );
}

#[tokio::test]
async fn test_callback_redirects_when_params_missing() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google/callback")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=invalid_request"
    );
}

#[tokio::test]
async fn test_callback_rejects_missing_state_cookie() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google/callback?code=sample-code&state=sample-state")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=state_missing"
    );
}

#[tokio::test]
async fn test_callback_rejects_state_mismatch() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google/callback?code=sample-code&state=incoming-state")
        .header("cookie", "g_oauth_state=different-cookie-state")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=state_mismatch"
    );
}

#[tokio::test]
async fn test_redirect_route_handles_request() {
    let router = test_router(None);
    let request = http::Request::builder()
        .method("GET")
        .uri("/auth/google/redirect")
        .body(topcoat::router::Body::empty())
        .unwrap();

    let response = router.handle(request).await;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=invalid_request"
    );
}

#[test]
fn parse_with_custom_endpoint_and_schemeless_base_url() {
    let config = GoogleOAuthConfig::parse_with_endpoint(
        Some("client-id-123".into()),
        Some("client-secret-456".into()),
        None,
        Some("192.168.0.50.nip.io:8080".into()),
        Some("/auth/google/redirect".into()),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        config.redirect_uri,
        "http://192.168.0.50.nip.io:8080/auth/google/redirect"
    );
}
