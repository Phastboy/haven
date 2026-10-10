use super::*;
use http::{Request, StatusCode};
use topcoat::{
    cookie::RouterBuilderCookieExt,
    router::{Body, Router, error::unauthorized},
    session::RouterBuilderSessionExt,
};

#[test]
fn default_cookie_name_is_sid() {
    let store = AdaptiveCookieTokenStore::new();
    assert_eq!(store.name, "sid");
    assert_eq!(store.prefixed_name(), "__Host-sid");
}

#[test]
fn custom_cookie_name() {
    let store = AdaptiveCookieTokenStore::new().name("custom_session");
    assert_eq!(store.name, "custom_session");
    assert_eq!(store.prefixed_name(), "__Host-custom_session");
}

#[test]
fn is_secure_evaluates_env_and_headers() {
    // 1. Explicit COOKIE_SECURE override
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            cookie_secure_env: Some("true"),
            ..Default::default()
        }
    ));
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            cookie_secure_env: Some("1"),
            ..Default::default()
        }
    ));
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            cookie_secure_env: Some("false"),
            trust_forwarded_proto_env: Some("true"),
            forwarded_proto_header: Some("https"),
            uri_scheme: Some("https"),
            public_base_url_env: Some("https://foo.com"),
            google_redirect_uri_env: Some("https://bar.com"),
        }
    ));
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            cookie_secure_env: Some("0"),
            trust_forwarded_proto_env: Some("true"),
            forwarded_proto_header: Some("https"),
            uri_scheme: Some("https"),
            public_base_url_env: Some("https://foo.com"),
            google_redirect_uri_env: Some("https://bar.com"),
        }
    ));

    // 2. X-Forwarded-Proto header (only when TRUST_FORWARDED_PROTO=true)
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            trust_forwarded_proto_env: Some("true"),
            forwarded_proto_header: Some("https"),
            ..Default::default()
        }
    ));
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            trust_forwarded_proto_env: Some("1"),
            forwarded_proto_header: Some("HTTPS"),
            ..Default::default()
        }
    ));
    // Untrusted proxy headers must NOT be honored
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            forwarded_proto_header: Some("https"),
            ..Default::default()
        }
    ));
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            trust_forwarded_proto_env: Some("false"),
            forwarded_proto_header: Some("https"),
            ..Default::default()
        }
    ));
}

#[test]
fn is_secure_evaluates_urls_and_fallback() {
    // 1. URI scheme
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            uri_scheme: Some("https"),
            ..Default::default()
        }
    ));

    // 2. PUBLIC_BASE_URL
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            public_base_url_env: Some("https://haven.example.com"),
            ..Default::default()
        }
    ));
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            public_base_url_env: Some("http://192.168.0.50.nip.io:8080"),
            ..Default::default()
        }
    ));
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            public_base_url_env: Some("192.168.0.50.nip.io:8080"),
            ..Default::default()
        }
    ));

    // 3. GOOGLE_REDIRECT_URI
    assert!(AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs {
            google_redirect_uri_env: Some("https://haven.example.com/auth/google/callback"),
            ..Default::default()
        }
    ));

    // 4. Default fallback
    assert!(!AdaptiveCookieTokenStore::is_secure(
        TransportSecurityInputs::default()
    ));
}

#[tokio::test]
async fn writes_insecure_cookie_when_http() {
    let store = AdaptiveCookieTokenStore::new();
    let config = topcoat::session::SessionConfig::builder()
        .token_store(store)
        .build();

    #[topcoat::router::page("/login-http")]
    async fn login_handler(cx: &Cx) -> topcoat::Result<()> {
        topcoat::session::start(cx).await?;
        Ok(())
    }

    let router = Router::builder()
        .cookies()
        .sessions(config)
        .page(login_handler)
        .build();

    let req = Request::builder()
        .method("GET")
        .uri("http://192.168.0.50.nip.io:8080/login-http")
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("should set cookie")
        .to_str()
        .unwrap();

    // Plain HTTP must NOT use __Host- prefix and must NOT have Secure flag
    assert!(set_cookie.starts_with("sid="), "cookie was: {set_cookie}");
    assert!(
        !set_cookie.contains("Secure"),
        "should not have Secure over HTTP: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Path=/"));
}

#[tokio::test]
async fn writes_secure_cookie_when_https() {
    let store = AdaptiveCookieTokenStore::new();
    let config = topcoat::session::SessionConfig::builder()
        .token_store(store)
        .build();

    #[topcoat::router::page("/login-secure")]
    async fn login_handler(cx: &Cx) -> topcoat::Result<()> {
        topcoat::session::start(cx).await?;
        Ok(())
    }

    let router = Router::builder()
        .cookies()
        .sessions(config)
        .page(login_handler)
        .build();

    let req = Request::builder()
        .method("GET")
        .uri("https://example.com/login-secure")
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = res
        .headers()
        .get("set-cookie")
        .expect("should set cookie")
        .to_str()
        .unwrap();

    // HTTPS must use __Host- prefix and have Secure flag
    assert!(
        set_cookie.starts_with("__Host-sid="),
        "cookie was: {set_cookie}"
    );
    assert!(
        set_cookie.contains("Secure"),
        "must have Secure on https: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("Path=/"));
}

#[tokio::test]
async fn reads_cookies_respecting_transport_security() {
    let store = AdaptiveCookieTokenStore::new();
    let config = topcoat::session::SessionConfig::builder()
        .token_store(store)
        .build();

    #[topcoat::router::page("/check-token")]
    async fn check_handler(cx: &Cx) -> topcoat::Result<()> {
        if topcoat::session::token_hash(cx).await?.is_some() {
            Ok(())
        } else {
            Err(unauthorized().into())
        }
    }

    let router = Router::builder()
        .cookies()
        .sessions(config)
        .page(check_handler)
        .build();

    let token = Token::random();
    let encoded_token = token.encode();

    // 1. Plain HTTP allows fallback to plain sid
    let http_plain_response = router
        .handle(
            Request::builder()
                .method("GET")
                .uri("http://192.168.0.50.nip.io:8080/check-token")
                .header("cookie", format!("sid={encoded_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(http_plain_response.status(), StatusCode::OK);

    // 2. Plain HTTP also accepts __Host-sid
    let http_host_response = router
        .handle(
            Request::builder()
                .method("GET")
                .uri("http://192.168.0.50.nip.io:8080/check-token")
                .header("cookie", format!("__Host-sid={encoded_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(http_host_response.status(), StatusCode::OK);

    // 3. Secure HTTPS accepts __Host-sid
    let secure_host_response = router
        .handle(
            Request::builder()
                .method("GET")
                .uri("https://example.com/check-token")
                .header("cookie", format!("__Host-sid={encoded_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(secure_host_response.status(), StatusCode::OK);

    // 4. Secure HTTPS rejects plain sid fallback (anti-cookie-tossing)
    let secure_plain_response = router
        .handle(
            Request::builder()
                .method("GET")
                .uri("https://example.com/check-token")
                .header("cookie", format!("sid={encoded_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(secure_plain_response.status(), StatusCode::UNAUTHORIZED);

    // 5. Request without cookie is rejected with 401
    let empty_response = router
        .handle(
            Request::builder()
                .method("GET")
                .uri("http://192.168.0.50.nip.io:8080/check-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(empty_response.status(), StatusCode::UNAUTHORIZED);
}
