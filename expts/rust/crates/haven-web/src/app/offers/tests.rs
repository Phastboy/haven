#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::similar_names,
    clippy::indexing_slicing,
    clippy::unimplemented,
    clippy::panic,
    reason = "test assertions and mocks"
)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use haven_domain::account::*;
use haven_domain::fakes::FakeOfferRepository;
use haven_domain::offer::*;
use haven_domain::ports::*;
use haven_domain::session::*;
use haven_domain::user::*;
use http::Request;
use topcoat::cookie::RouterBuilderCookieExt;
use topcoat::router::{Body, RouterBuilderDiscoverExt, to_bytes};
use topcoat::session::{RouterBuilderSessionExt, SessionConfig, Token};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct TestUserRepository {
    pub users_by_token: Arc<Mutex<HashMap<String, User>>>,
}

#[async_trait]
impl UserRepository for TestUserRepository {
    async fn find_by_account(&self, _account_id: AccountId) -> Result<Option<User>, RepoError> {
        unimplemented!()
    }
    async fn create(&self, _account_id: AccountId) -> Result<User, RepoError> {
        unimplemented!()
    }
    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError> {
        let lock = self.users_by_token.lock().unwrap();
        Ok(lock.get(token_hash.as_str()).cloned())
    }
}

pub struct TestRegistry {
    pub offers_repo: FakeOfferRepository,
    pub users_repo: TestUserRepository,
}

impl Registry for TestRegistry {
    fn offers(&self) -> &(dyn OfferRepository + 'static) {
        &self.offers_repo
    }
    fn accounts(&self) -> &(dyn AccountRepository + 'static) {
        unimplemented!()
    }
    fn users(&self) -> &(dyn UserRepository + 'static) {
        &self.users_repo
    }
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static) {
        unimplemented!()
    }
    fn sessions(&self) -> &(dyn SessionRepository + 'static) {
        unimplemented!()
    }
}

fn setup_test_app() -> (topcoat::router::Router, TestRegistry) {
    let registry = TestRegistry {
        offers_repo: FakeOfferRepository::default(),
        users_repo: TestUserRepository::default(),
    };

    let state = crate::app::state::AppState {
        registry: Arc::new(TestRegistry {
            offers_repo: registry.offers_repo.clone(),
            users_repo: registry.users_repo.clone(),
        }),
        google_oauth: None,
        http_client: reqwest::Client::new(),
    };

    let router = crate::app::router()
        .cookies()
        .sessions(
            SessionConfig::builder()
                .token_store(crate::token_store::AdaptiveCookieTokenStore::new().name("sid"))
                .build(),
        )
        .app_context(state)
        .app_context(crate::rate_limit::SignInLimiter::new())
        .app_context(crate::rate_limit::CreateOfferLimiter::new())
        .discover()
        .build();

    (router, registry)
}

fn create_auth_session(users_repo: &TestUserRepository, user: User) -> String {
    let token = Token::random();
    let encoded = token.encode();
    let hash_hex = crate::app::auth::guard::token_hash_hex(&token.hash());
    users_repo
        .users_by_token
        .lock()
        .unwrap()
        .insert(hash_hex, user);
    encoded
}

async fn body_to_string(body: Body) -> String {
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn test_unauthenticated_requests_refused_for_every_manage_route() {
    let (router, _) = setup_test_app();

    let dummy_id = Uuid::new_v4();
    let manage_routes = [
        ("GET", "/offers/manage".to_string()),
        ("GET", "/offers/manage/new".to_string()),
        ("POST", "/offers/manage/new".to_string()),
        ("GET", format!("/offers/manage/{dummy_id}")),
        ("GET", format!("/offers/manage/{dummy_id}/edit")),
        ("POST", format!("/offers/manage/{dummy_id}/edit")),
        ("POST", format!("/offers/manage/{dummy_id}/delete")),
    ];

    for (method, uri) in &manage_routes {
        let req = Request::builder()
            .method(*method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let response = router.handle(req).await;

        // Every owner route must refuse unauthenticated requests
        assert!(
            response.status().is_redirection()
                || response.status() == http::StatusCode::UNAUTHORIZED,
            "Route {method} {uri} did not refuse unauthenticated request! Got status: {}",
            response.status()
        );

        if response.status().is_redirection() {
            let location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert_eq!(
                location, "/auth/sign-in",
                "Route {method} {uri} redirected to {location} instead of /auth/sign-in"
            );
        }

        // Cache-Control must be private, no-store
        let cache_control = response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            cache_control.contains("private") && cache_control.contains("no-store"),
            "Route {method} {uri} missing private, no-store cache header! Got: {cache_control}"
        );
    }
}

#[tokio::test]
async fn test_public_slug_route_caching_and_headers() {
    let (router, registry) = setup_test_app();

    // Create an offer in the repo
    let user_id = UserId::new();
    let create_offer = CreateOffer {
        title: "Vintage Oak Table".to_string(),
        description: "A beautifully restored vintage oak dining table.".to_string(),
        price: Price::new(250).unwrap(),
        currency: CurrencyCode::parse("USD").unwrap(),
    };
    let offer = registry
        .offers_repo
        .create(user_id, IdempotencyKey(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    let slug_url = format!("/offers/{}", offer.slug);

    // 1. Public request without any session
    let req = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    // Verify cache headers
    let cc = res
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        cc.contains("public"),
        "Public page must have public cache policy"
    );

    let etag = res
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .expect("Public page must supply ETag")
        .to_string();

    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("Vintage Oak Table"));
    assert!(body.contains("250 USD"));
    assert!(body.contains("A beautifully restored vintage oak dining table."));
    assert!(body.contains(&format!("href=\"/offers/{}\"", offer.slug)));
    assert!(!body.contains("action=\"/offers/manage"));
    assert!(!body.contains("Edit"));
    assert!(!body.contains("Delete"));

    // 2. 304 Not Modified when If-None-Match matches ETag (including weak, wildcard, and comma-separated)
    assert!(
        etag.contains(env!("CARGO_PKG_VERSION")),
        "ETag must include the application build/version token"
    );

    let req_304 = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("if-none-match", &etag)
        .body(Body::empty())
        .unwrap();
    let res_304 = router.handle(req_304).await;
    assert_eq!(res_304.status(), http::StatusCode::NOT_MODIFIED);

    // Wildcard match
    let req_wildcard = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("if-none-match", "*")
        .body(Body::empty())
        .unwrap();
    let res_wildcard = router.handle(req_wildcard).await;
    assert_eq!(res_wildcard.status(), http::StatusCode::NOT_MODIFIED);

    // Weak match
    let weak_etag = format!("W/{etag}");
    let req_weak = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("if-none-match", &weak_etag)
        .body(Body::empty())
        .unwrap();
    let res_weak = router.handle(req_weak).await;
    assert_eq!(res_weak.status(), http::StatusCode::NOT_MODIFIED);

    // Comma-separated list with weak match and trimming
    let list_inm = format!("\"random-tag\", {weak_etag}");
    let req_list = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("if-none-match", &list_inm)
        .body(Body::empty())
        .unwrap();
    let res_list = router.handle(req_list).await;
    assert_eq!(res_list.status(), http::StatusCode::NOT_MODIFIED);

    // Non-matching tag returns 200
    let req_mismatch = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("if-none-match", "\"unrelated-tag\"")
        .body(Body::empty())
        .unwrap();
    let res_mismatch = router.handle(req_mismatch).await;
    assert_eq!(res_mismatch.status(), http::StatusCode::OK);

    // 3. 404 for unknown slug
    let req_404 = Request::builder()
        .method("GET")
        .uri("/offers/nonexistent-item-12345678")
        .body(Body::empty())
        .unwrap();

    let res_404 = router.handle(req_404).await;
    assert_eq!(res_404.status(), http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_robots_txt_disallows_manage() {
    let (router, _) = setup_test_app();

    let req = Request::builder()
        .method("GET")
        .uri("/robots.txt")
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    let body = body_to_string(res.into_body()).await;
    assert!(body.contains("User-agent: *"));
    assert!(body.contains("Disallow: /offers/manage/"));
}

#[tokio::test]
async fn test_offers_new_redirects_to_manage_new() {
    let (router, _) = setup_test_app();

    let req = Request::builder()
        .method("GET")
        .uri("/offers/new")
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::SEE_OTHER);
    let loc = res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(loc, "/offers/manage/new");

    let post_req = Request::builder()
        .method("POST")
        .uri("/offers/new")
        .body(Body::empty())
        .unwrap();

    let post_res = router.handle(post_req).await;
    assert_eq!(post_res.status(), http::StatusCode::PERMANENT_REDIRECT);
    let post_loc = post_res
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(post_loc, "/offers/manage/new");
}

#[tokio::test]
async fn test_owner_routes_enforce_ownership_and_identical_404() {
    let (router, registry) = setup_test_app();

    let user1 = User {
        id: UserId::new(),
        account_id: AccountId::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let user2 = User {
        id: UserId::new(),
        account_id: AccountId::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let session1_token = create_auth_session(&registry.users_repo, user1.clone());
    let session2_token = create_auth_session(&registry.users_repo, user2.clone());

    // Create offer owned by user1
    let create_offer = CreateOffer {
        title: "User 1 Painting".to_string(),
        description: "Original acrylic on canvas by user 1.".to_string(),
        price: Price::ZERO,
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user1.id, IdempotencyKey(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // 1. User 1 accesses their own offer via ID route
    let req_owner = Request::builder()
        .method("GET")
        .uri(format!("/offers/manage/{}", offer.id))
        .header("cookie", format!("sid={session1_token}"))
        .body(Body::empty())
        .unwrap();

    let res_owner = router.handle(req_owner).await;
    assert_eq!(res_owner.status(), http::StatusCode::OK);

    let cc = res_owner
        .headers()
        .get("cache-control")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(cc.contains("private") && cc.contains("no-store"));

    let robots = res_owner
        .headers()
        .get("x-robots-tag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(robots.contains("noindex"));

    let body = body_to_string(res_owner.into_body()).await;
    assert!(body.contains("User 1 Painting"));
    assert!(body.contains(&format!("href=\"/offers/manage/{}/edit\"", offer.id)));
    assert!(body.contains(&format!("action=\"/offers/manage/{}/delete\"", offer.id)));
    assert!(body.contains(&format!("href=\"/offers/{}\"", offer.slug)));

    // 2. User 2 accesses User 1's offer via ID route (must return 404, identical to nonexistent)
    let req_not_owner = Request::builder()
        .method("GET")
        .uri(format!("/offers/manage/{}", offer.id))
        .header("cookie", format!("sid={session2_token}"))
        .body(Body::empty())
        .unwrap();

    let res_not_owner = router.handle(req_not_owner).await;
    assert_eq!(res_not_owner.status(), http::StatusCode::NOT_FOUND);

    // 3. User 2 accesses nonexistent UUID (must return 404)
    let non_existent_id = Uuid::new_v4();
    let req_nonexistent = Request::builder()
        .method("GET")
        .uri(format!("/offers/manage/{non_existent_id}"))
        .header("cookie", format!("sid={session2_token}"))
        .body(Body::empty())
        .unwrap();

    let res_nonexistent = router.handle(req_nonexistent).await;
    assert_eq!(res_nonexistent.status(), http::StatusCode::NOT_FOUND);

    // User 1's offers list only shows their offers
    let req_list = Request::builder()
        .method("GET")
        .uri("/offers/manage")
        .header("cookie", format!("sid={session1_token}"))
        .body(Body::empty())
        .unwrap();

    let res_list = router.handle(req_list).await;
    assert_eq!(res_list.status(), http::StatusCode::OK);
    let list_body = body_to_string(res_list.into_body()).await;
    assert!(list_body.contains("User 1 Painting"));
}

#[tokio::test]
async fn test_landing_page_renders_with_components() {
    let (router, _) = setup_test_app();

    let req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = body_to_string(res.into_body()).await;

    assert!(body.contains("Exchange offers directly"));
    assert!(body.contains("href=\"/offers\""));
    assert!(body.contains("class=\"btn btn-primary\""));
    assert!(body.contains("Browse Offers"));
    assert!(body.contains("href=\"/auth/sign-in\""));
    assert!(body.contains("class=\"btn btn-secondary\""));
    assert!(body.contains("Sign In"));
}

#[tokio::test]
async fn test_auth_pages_render_with_components_and_classes() {
    let (router, _) = setup_test_app();

    // 1. /auth/sign-in (defaults to email delivery disabled)
    let req_sign_in = Request::builder()
        .method("GET")
        .uri("/auth/sign-in")
        .body(Body::empty())
        .unwrap();

    let res_sign_in = router.handle(req_sign_in).await;
    assert_eq!(res_sign_in.status(), http::StatusCode::OK);
    let body_sign_in = body_to_string(res_sign_in.into_body()).await;
    assert!(body_sign_in.contains("class=\"auth-card\""));
    assert!(body_sign_in.contains("class=\"field-label\""));
    assert!(body_sign_in.contains("for=\"email\""));
    assert!(body_sign_in.contains("type=\"email\""));
    assert!(body_sign_in.contains("disabled=\"disabled\""));
    assert!(body_sign_in.contains("class=\"auth-notice\""));
    assert!(body_sign_in.contains("Email sign-in is temporarily unavailable"));
    assert!(body_sign_in.contains("Send Magic Link (Unavailable)"));

    // 2. /auth/sign-in with error query
    let req_sign_in_err = Request::builder()
        .method("GET")
        .uri("/auth/sign-in?error=google_cancelled")
        .body(Body::empty())
        .unwrap();

    let res_sign_in_err = router.handle(req_sign_in_err).await;
    assert_eq!(res_sign_in_err.status(), http::StatusCode::OK);
    let body_sign_in_err = body_to_string(res_sign_in_err.into_body()).await;
    assert!(body_sign_in_err.contains("class=\"form-error-summary\""));
    assert!(body_sign_in_err.contains("role=\"alert\""));
    assert!(body_sign_in_err.contains("Google sign-in was cancelled."));

    // 3. POST /auth/sign-in rejects submission when delivery is disabled
    let req_post_sign_in = Request::builder()
        .method("POST")
        .uri("/auth/sign-in")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("email=someone%40example.com"))
        .unwrap();
    let res_post = router.handle(req_post_sign_in).await;
    assert_eq!(res_post.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res_post
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in?error=email_delivery_unavailable"
    );

    // 4. /auth/sent
    let req_sent = Request::builder()
        .method("GET")
        .uri("/auth/sent")
        .body(Body::empty())
        .unwrap();

    let res_sent = router.handle(req_sent).await;
    assert_eq!(res_sent.status(), http::StatusCode::OK);
    let body_sent = body_to_string(res_sent.into_body()).await;
    assert!(body_sent.contains("class=\"auth-card\""));
    assert!(body_sent.contains("Email delivery unavailable"));
    assert!(body_sent.contains("href=\"/auth/sign-in\""));
    assert!(body_sent.contains("class=\"btn btn-secondary\""));

    // 5. /auth/verify
    let req_verify = Request::builder()
        .method("GET")
        .uri("/auth/verify?token=sample_token")
        .body(Body::empty())
        .unwrap();

    let res_verify = router.handle(req_verify).await;
    assert_eq!(res_verify.status(), http::StatusCode::OK);
    let body_verify = body_to_string(res_verify.into_body()).await;
    assert!(body_verify.contains("class=\"auth-card\""));
    assert!(body_verify.contains("Sign In Verification"));
    assert!(body_verify.contains("class=\"btn btn-primary\""));
    assert!(body_verify.contains("Confirm and Sign In"));
}
