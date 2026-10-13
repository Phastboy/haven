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
    async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>, RepoError> {
        let lock = self.users_by_token.lock().unwrap();
        for u in lock.values() {
            if u.id == user_id {
                return Ok(Some(u.clone()));
            }
        }
        Ok(None)
    }
    async fn find_by_handle(&self, handle: &ProfileHandle) -> Result<Option<User>, RepoError> {
        let lock = self.users_by_token.lock().unwrap();
        for u in lock.values() {
            if u.handle.as_str() == handle.as_str() {
                return Ok(Some(u.clone()));
            }
        }
        Ok(None)
    }
    async fn create(
        &self,
        _account_id: AccountId,
        _profile: &UserProfileUpdate,
    ) -> Result<User, RepoError> {
        unimplemented!()
    }
    async fn update_profile(
        &self,
        user_id: UserId,
        profile: &UserProfileUpdate,
    ) -> Result<User, RepoError> {
        let mut lock = self.users_by_token.lock().unwrap();
        for u in lock.values() {
            if u.id != user_id && u.handle.as_str() == profile.handle.as_str() {
                return Err(RepoError::Conflict {
                    entity: "user",
                    key: profile.handle.as_str().to_string(),
                });
            }
        }
        for u in lock.values_mut() {
            if u.id == user_id {
                u.display_name = profile.display_name.clone();
                u.handle = profile.handle.clone();
                u.bio = profile.bio.clone();
                u.updated_at = Utc::now();
                return Ok(u.clone());
            }
        }
        Err(RepoError::NotFound {
            entity: "user",
            id: user_id.to_string(),
        })
    }
    async fn update_avatar(
        &self,
        user_id: UserId,
        avatar: Option<&haven_domain::image::ImagePublicId>,
    ) -> Result<User, RepoError> {
        let mut lock = self.users_by_token.lock().unwrap();
        for u in lock.values_mut() {
            if u.id == user_id {
                u.avatar_public_id = avatar.cloned();
                u.updated_at = Utc::now();
                return Ok(u.clone());
            }
        }
        Err(RepoError::NotFound {
            entity: "user",
            id: user_id.to_string(),
        })
    }
    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError> {
        let lock = self.users_by_token.lock().unwrap();
        Ok(lock.get(token_hash.as_str()).cloned())
    }
}

fn mock_user(id: UserId, account_id: AccountId, handle_str: &str) -> User {
    User {
        id,
        account_id,
        display_name: DisplayName::parse(handle_str).unwrap(),
        handle: ProfileHandle::parse(handle_str).unwrap(),
        avatar_public_id: None,
        bio: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
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
    fn offer_images(&self) -> &(dyn OfferImageRepository + 'static) {
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
        email_delivery_enabled: Some(false),
        image_storage: Arc::new(crate::storage::MockImageStorage::new()),
    };

    let origins = crate::app::origin::compute_trusted_origins("3000", None, None);
    let origin_policy = topcoat::router::OriginPolicy::new().trust_origins(origins);
    let router = crate::app::router()
        .origin_policy(origin_policy)
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
        ("GET", "/profile/offers".to_string()),
        ("GET", "/profile/offers/new".to_string()),
        ("POST", "/profile/offers/new".to_string()),
        ("GET", format!("/profile/offers/{dummy_id}")),
        ("GET", format!("/profile/offers/{dummy_id}/edit")),
        ("POST", format!("/profile/offers/{dummy_id}/edit")),
        ("POST", format!("/profile/offers/{dummy_id}")),
        ("GET", "/profile/account".to_string()),
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
    let user_id = UserId::generate();
    let create_offer = CreateOffer {
        title: "Vintage Oak Table".to_string(),
        description: "A beautifully restored vintage oak dining table.".to_string(),
        price: Price::new(250).unwrap(),
        currency: CurrencyCode::parse("USD").unwrap(),
    };
    let offer = registry
        .offers_repo
        .create(user_id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
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
    assert!(body.contains("Disallow: /profile/"));
}

#[tokio::test]
async fn test_pruned_endpoints_return_not_found() {
    let (router, _) = setup_test_app();

    for uri in ["/offers/new", "/profiles", "/offers/manage"] {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let res = router.handle(req).await;
        assert_eq!(
            res.status(),
            http::StatusCode::NOT_FOUND,
            "Endpoint {uri} was not pruned to 404"
        );
    }
}

#[tokio::test]
async fn test_owner_routes_enforce_ownership_and_identical_404() {
    let (router, registry) = setup_test_app();

    let user1 = mock_user(UserId::generate(), AccountId::generate(), "user-one");
    let user2 = mock_user(UserId::generate(), AccountId::generate(), "user-two");

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
        .create(user1.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // 1. User 1 accesses their own offer via ID route
    let req_owner = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
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
    assert!(body.contains(&format!("href=\"/profile/offers/{}/edit\"", offer.id)));
    assert!(body.contains("popovertarget=\"delete-offer-dialog\""));
    assert!(body.contains("<dialog id=\"delete-offer-dialog\" popover=\"auto\""));
    assert!(body.contains(&format!("action=\"/profile/offers/{}\"", offer.id)));
    assert!(body.contains(&format!("href=\"/offers/{}\"", offer.slug)));

    // 2. User 2 accesses User 1's offer via ID route (must return 404, identical to nonexistent)
    let req_not_owner = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session2_token}"))
        .body(Body::empty())
        .unwrap();

    let res_not_owner = router.handle(req_not_owner).await;
    assert_eq!(res_not_owner.status(), http::StatusCode::NOT_FOUND);

    // 3. User 2 accesses nonexistent UUID (must return 404)
    let non_existent_id = Uuid::new_v4();
    let req_nonexistent = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{non_existent_id}"))
        .header("cookie", format!("sid={session2_token}"))
        .body(Body::empty())
        .unwrap();

    let res_nonexistent = router.handle(req_nonexistent).await;
    assert_eq!(res_nonexistent.status(), http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_owner_offers_quick_delete_and_navigation_loop_guard() {
    let (router, registry) = setup_test_app();

    let user1 = mock_user(UserId::generate(), AccountId::generate(), "user-one");
    let session1_token = create_auth_session(&registry.users_repo, user1.clone());

    let create_offer = CreateOffer {
        title: "User 1 Painting".to_string(),
        description: "Original acrylic on canvas by user 1.".to_string(),
        price: Price::ZERO,
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user1.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // User 1's offers list shows their offers and has inline quick delete dialog
    let req_list = Request::builder()
        .method("GET")
        .uri("/profile/offers")
        .header("host", "localhost:3000")
        .header("cookie", format!("sid={session1_token}"))
        .header(
            "referer",
            format!("http://localhost:3000/profile/offers/{}", offer.id),
        )
        .body(Body::empty())
        .unwrap();

    let res_list = router.handle(req_list).await;
    assert_eq!(res_list.status(), http::StatusCode::OK);
    let list_body = body_to_string(res_list.into_body()).await;
    assert!(list_body.contains("User 1 Painting"));
    // Quick delete affordance present directly on listing
    assert!(list_body.contains(&format!("action=\"/profile/offers/{}\"", offer.id)));
    assert!(list_body.contains(&format!("confirm-del-profile-offers-{}", offer.id)));
    // Navigation loop guard: coming up from /profile/offers/{id} must point back up to /profile, not loop back to {id}
    assert!(list_body.contains("href=\"/profile\""));

    // User 1 deletes their offer via POST /profile/offers/{id}
    let req_delete = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session1_token}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap();
    let res_delete = router.handle(req_delete).await;
    assert_eq!(res_delete.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res_delete
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/profile/offers"
    );

    // After deletion, offer is gone (returns 404)
    let req_deleted_check = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session1_token}"))
        .body(Body::empty())
        .unwrap();
    let res_deleted_check = router.handle(req_deleted_check).await;
    assert_eq!(res_deleted_check.status(), http::StatusCode::NOT_FOUND);
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

#[tokio::test]
async fn test_origin_policy_allows_trusted_origin_and_rejects_untrusted() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "user-origin");
    let session_token = create_auth_session(&registry.users_repo, user);

    // 1. Untrusted origin on state-changing POST must receive 403 Forbidden
    let req_untrusted = Request::builder()
        .method("POST")
        .uri("/profile/offers/new")
        .header("cookie", format!("sid={session_token}"))
        .header("origin", "https://evil.example.com")
        .header("sec-fetch-site", "cross-site")
        .body(Body::empty())
        .unwrap();

    let res_untrusted = router.handle(req_untrusted).await;
    assert_eq!(res_untrusted.status(), http::StatusCode::FORBIDDEN);

    // 2. Trusted origin on state-changing POST passes OriginLayer (not 403)
    let req_trusted = Request::builder()
        .method("POST")
        .uri("/profile/offers/new")
        .header("cookie", format!("sid={session_token}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap();

    let res_trusted = router.handle(req_trusted).await;
    assert_ne!(res_trusted.status(), http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_offer_ownership_checked_before_invalid_form() {
    let (router, registry) = setup_test_app();

    let user1 = mock_user(UserId::generate(), AccountId::generate(), "user-one");
    let user2 = mock_user(UserId::generate(), AccountId::generate(), "user-two");

    let _session1_token = create_auth_session(&registry.users_repo, user1.clone());
    let session2_token = create_auth_session(&registry.users_repo, user2);

    let create_offer = CreateOffer {
        title: "User 1 Item".to_string(),
        description: "User 1 item description.".to_string(),
        price: Price::ZERO,
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user1.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // User 2 sends empty/missing form to User 1's offer edit route.
    // Must return 404 Not Found (ownership check first), NOT 422 Unprocessable Entity!
    let req_unauthorized = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/edit", offer.id))
        .header("cookie", format!("sid={session2_token}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap();

    let res_unauthorized = router.handle(req_unauthorized).await;
    assert_eq!(res_unauthorized.status(), http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_public_profile_route_renders_maker_and_offers_and_404_when_missing() {
    let (router, registry) = setup_test_app();

    // 1. Unknown handle returns 404
    let req_missing = Request::builder()
        .method("GET")
        .uri("/profiles/unknown-handle")
        .body(Body::empty())
        .unwrap();
    let res_missing = router.handle(req_missing).await;
    assert_eq!(res_missing.status(), http::StatusCode::NOT_FOUND);

    // 2. Known user with profile renders display name, handle, and empty state
    let user = mock_user(UserId::generate(), AccountId::generate(), "alice-maker");
    let _ = create_auth_session(&registry.users_repo, user.clone());

    let req_user = Request::builder()
        .method("GET")
        .uri("/profiles/alice-maker")
        .body(Body::empty())
        .unwrap();
    let res_user = router.handle(req_user).await;
    assert_eq!(res_user.status(), http::StatusCode::OK);
    let body = body_to_string(res_user.into_body()).await;
    assert!(body.contains("alice-maker"));
    assert!(body.contains("@alice-maker"));
    assert!(body.contains("No offers yet"));

    // 3. Known user with an offer renders the offer card
    let create_offer = CreateOffer {
        title: "Handmade Wooden Chair".to_string(),
        description: "Solid oak handcrafted dining chair.".to_string(),
        price: Price::new(15000).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    let req_with_offer = Request::builder()
        .method("GET")
        .uri("/profiles/alice-maker")
        .body(Body::empty())
        .unwrap();
    let res_with_offer = router.handle(req_with_offer).await;
    assert_eq!(res_with_offer.status(), http::StatusCode::OK);
    let body_offer = body_to_string(res_with_offer.into_body()).await;
    assert!(body_offer.contains("Handmade Wooden Chair"));
    assert!(body_offer.contains("15000 NGN"));
}

#[tokio::test]
async fn test_owner_profile_route_requires_auth_and_allows_editing() {
    let (router, registry) = setup_test_app();

    // 1. Unauthenticated GET redirects to /auth/sign-in
    let req_unauth = Request::builder()
        .method("GET")
        .uri("/profile")
        .body(Body::empty())
        .unwrap();
    let res_unauth = router.handle(req_unauth).await;
    assert!(res_unauth.status().is_redirection());

    // 2. Authenticated GET /profile renders dashboard view (not edit form)
    let user = mock_user(UserId::generate(), AccountId::generate(), "alex-wood");
    let session_token = create_auth_session(&registry.users_repo, user.clone());

    let req_auth = Request::builder()
        .method("GET")
        .uri("/profile")
        .header("cookie", format!("sid={session_token}"))
        .body(Body::empty())
        .unwrap();
    let res_auth = router.handle(req_auth).await;
    assert_eq!(res_auth.status(), http::StatusCode::OK);
    let body_get = body_to_string(res_auth.into_body()).await;
    // Renders identity and dashboard sections
    assert!(body_get.contains("alex-wood"));
    assert!(body_get.contains("@alex-wood"));
    assert!(body_get.contains("/profile/edit"));
    assert!(body_get.contains("Your Offers"));
    assert!(body_get.contains("href=\"/profile/offers\""));
    assert!(body_get.contains("href=\"/profile/account\""));
    // Header only has clean nav (not cluttered with Sign Out form or Your Offers)
    assert!(body_get.contains("<a href=\"/profile\">Profile</a>"));

    // 2b. Authenticated GET /profile/account renders account section with sign out
    let req_account = Request::builder()
        .method("GET")
        .uri("/profile/account")
        .header("cookie", format!("sid={session_token}"))
        .body(Body::empty())
        .unwrap();
    let res_account = router.handle(req_account).await;
    assert_eq!(res_account.status(), http::StatusCode::OK);
    let body_account = body_to_string(res_account.into_body()).await;
    assert!(body_account.contains("Account &amp; Session"));
    assert!(body_account.contains("Sign Out"));
    assert!(body_account.contains("action=\"/auth/sign-out\""));

    // 3. Authenticated GET /profile/edit renders edit form
    let req_edit = Request::builder()
        .method("GET")
        .uri("/profile/edit")
        .header("cookie", format!("sid={session_token}"))
        .body(Body::empty())
        .unwrap();
    let res_edit = router.handle(req_edit).await;
    assert_eq!(res_edit.status(), http::StatusCode::OK);
    let body_edit = body_to_string(res_edit.into_body()).await;
    assert!(body_edit.contains("Edit Profile"));
    assert!(body_edit.contains("value=\"alex-wood\""));

    // 4. Authenticated POST /profile/edit updates profile and redirects to /profile?updated=1
    let post_body = "display_name=Alexander+Wood&handle=alex-crafts&bio=Woodworking+artisan.";
    let req_post = Request::builder()
        .method("POST")
        .uri("/profile/edit")
        .header("cookie", format!("sid={session_token}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(post_body))
        .unwrap();
    let res_post = router.handle(req_post).await;
    assert_eq!(res_post.status(), http::StatusCode::SEE_OTHER);
    let location = res_post
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(location, "/profile?updated=1");

    // 5. Verify user was updated
    let updated_user = registry
        .users_repo
        .find_by_handle(&ProfileHandle::parse("alex-crafts").unwrap())
        .await
        .unwrap();
    assert!(updated_user.is_some());
    let u = updated_user.unwrap();
    assert_eq!(u.display_name.as_str(), "Alexander Wood");
    assert_eq!(u.bio.as_ref().unwrap().as_str(), "Woodworking artisan.");
}

#[tokio::test]
async fn test_owner_profile_rejects_duplicate_handle() {
    let (router, registry) = setup_test_app();

    let user1 = mock_user(UserId::generate(), AccountId::generate(), "user-one");
    let user2 = mock_user(UserId::generate(), AccountId::generate(), "user-two");
    let _ = create_auth_session(&registry.users_repo, user1);
    let session2 = create_auth_session(&registry.users_repo, user2);

    // User 2 tries to take "user-one"
    let post_body = "display_name=Second+User&handle=user-one&bio=";
    let req_post = Request::builder()
        .method("POST")
        .uri("/profile/edit")
        .header("cookie", format!("sid={session2}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(post_body))
        .unwrap();
    let res_post = router.handle(req_post).await;
    assert_eq!(res_post.status(), http::StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_to_string(res_post.into_body()).await;
    assert!(body.contains("already taken"));
}

#[tokio::test]
async fn test_maker_attribution_rendered_on_offer_page_and_feed() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "artisan-bob");
    let _ = create_auth_session(&registry.users_repo, user.clone());

    let create_offer = CreateOffer {
        title: "Handmade Ceramic Vase".to_string(),
        description: "Stoneware vase fired in wood kiln.".to_string(),
        price: Price::new(4500).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // Check /offers/{slug}
    let req_slug = Request::builder()
        .method("GET")
        .uri(format!("/offers/{}", offer.slug))
        .body(Body::empty())
        .unwrap();
    let res_slug = router.handle(req_slug).await;
    assert_eq!(res_slug.status(), http::StatusCode::OK);
    let body_slug = body_to_string(res_slug.into_body()).await;
    assert!(body_slug.contains("Offered by"));
    assert!(body_slug.contains("/profiles/artisan-bob"));

    // Check feed /offers
    let req_feed = Request::builder()
        .method("GET")
        .uri("/offers")
        .body(Body::empty())
        .unwrap();
    let res_feed = router.handle(req_feed).await;
    assert_eq!(res_feed.status(), http::StatusCode::OK);
    let body_feed = body_to_string(res_feed.into_body()).await;
    assert!(body_feed.contains("Handmade Ceramic Vase"));
    assert!(body_feed.contains("/profiles/test-maker"));
}

#[tokio::test]
async fn test_navigation_layout_rule_main_vs_child() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "artisan-nav");
    let _ = create_auth_session(&registry.users_repo, user.clone());
    let create_offer = CreateOffer {
        title: "Carved Wooden Bowl".to_string(),
        description: "Hand-carved wooden bowl from walnut.".to_string(),
        price: Price::new(3000).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    // 1. Top-level route /offers uses Main layout
    let req_main = Request::builder()
        .method("GET")
        .uri("/offers")
        .body(Body::empty())
        .unwrap();
    let res_main = router.handle(req_main).await;
    assert_eq!(res_main.status(), http::StatusCode::OK);
    let body_main = body_to_string(res_main.into_body()).await;
    assert!(body_main.contains("layout-main"));
    assert!(!body_main.contains("layout-child"));
    assert!(!body_main.contains("child-header"));

    // 2. Child route /offers/{slug} with no Referer uses Child layout with fallback parent
    let req_child_no_ref = Request::builder()
        .method("GET")
        .uri(format!("/offers/{}", offer.slug))
        .body(Body::empty())
        .unwrap();
    let res_child_no_ref = router.handle(req_child_no_ref).await;
    assert_eq!(res_child_no_ref.status(), http::StatusCode::OK);
    let body_child_no_ref = body_to_string(res_child_no_ref.into_body()).await;
    assert!(body_child_no_ref.contains("layout-child"));
    assert!(body_child_no_ref.contains("site-header desktop-only"));
    assert!(body_child_no_ref.contains("child-header"));
    assert!(body_child_no_ref.contains("href=\"/offers\""));

    // 3. Child route with same-site Referer returns previous page
    let req_child_with_ref = Request::builder()
        .method("GET")
        .uri(format!("/offers/{}", offer.slug))
        .header("host", "localhost:3000")
        .header("referer", "http://localhost:3000/offers?sort=newest")
        .body(Body::empty())
        .unwrap();
    let res_child_with_ref = router.handle(req_child_with_ref).await;
    assert_eq!(res_child_with_ref.status(), http::StatusCode::OK);
    let body_child_with_ref = body_to_string(res_child_with_ref.into_body()).await;
    assert!(body_child_with_ref.contains("href=\"/offers?sort=newest\""));

    // 4. Child route with self-referencing Referer (refresh) guards against loop
    let req_child_refresh = Request::builder()
        .method("GET")
        .uri(format!("/offers/{}", offer.slug))
        .header("host", "localhost:3000")
        .header(
            "referer",
            format!("http://localhost:3000/offers/{}", offer.slug),
        )
        .body(Body::empty())
        .unwrap();
    let res_child_refresh = router.handle(req_child_refresh).await;
    assert_eq!(res_child_refresh.status(), http::StatusCode::OK);
    let body_child_refresh = body_to_string(res_child_refresh.into_body()).await;
    assert!(body_child_refresh.contains("href=\"/offers\""));

    // 5. Back button uses an explicit SVG icon, not a unicode text sign
    assert!(body_child_refresh.contains("back-icon-svg"));
    assert!(body_child_refresh.contains("<svg"));

    // 6. Public profile navigated from /profile returns to /profile via Referer
    let session = create_auth_session(&registry.users_repo, user.clone());
    let req_profile_pub_ref = Request::builder()
        .method("GET")
        .uri(format!("/profiles/{}", user.handle.as_str()))
        .header("host", "localhost:3000")
        .header("cookie", format!("sid={session}"))
        .header("referer", "http://localhost:3000/profile")
        .body(Body::empty())
        .unwrap();
    let res_profile_pub_ref = router.handle(req_profile_pub_ref).await;
    assert_eq!(res_profile_pub_ref.status(), http::StatusCode::OK);
    // Security layer sets Referrer-Policy: same-origin
    assert_eq!(
        res_profile_pub_ref
            .headers()
            .get("referrer-policy")
            .and_then(|v| v.to_str().ok()),
        Some("same-origin")
    );
    let body_profile_pub_ref = body_to_string(res_profile_pub_ref.into_body()).await;
    assert!(body_profile_pub_ref.contains("href=\"/profile\""));
    assert!(body_profile_pub_ref.contains("back-icon-svg"));

    // 7. Public profile viewed directly by owner falls back to /profile (parent)
    let req_profile_pub_no_ref = Request::builder()
        .method("GET")
        .uri(format!("/profiles/{}", user.handle.as_str()))
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();
    let res_profile_pub_no_ref = router.handle(req_profile_pub_no_ref).await;
    assert_eq!(res_profile_pub_no_ref.status(), http::StatusCode::OK);
    let body_profile_pub_no_ref = body_to_string(res_profile_pub_no_ref.into_body()).await;
    assert!(body_profile_pub_no_ref.contains("href=\"/profile\""));
}

#[tokio::test]
async fn test_public_offer_back_returns_to_owner_offer_page() {
    let (router, registry) = setup_test_app();
    let user = mock_user(UserId::generate(), AccountId::generate(), "artisan");
    let session = create_auth_session(&registry.users_repo, user.clone());

    let create_offer = CreateOffer {
        title: "Handcrafted Desk".to_string(),
        description: "Solid oak study desk.".to_string(),
        price: Price::ZERO,
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();

    let owner_page_url = format!("http://localhost:3000/profile/offers/{}", offer.id);

    // Navigating to public offer page from owner manage view
    let req = Request::builder()
        .method("GET")
        .uri(format!("/offers/{}", offer.slug))
        .header("host", "localhost:3000")
        .header("cookie", format!("sid={session}"))
        .header("referer", &owner_page_url)
        .body(Body::empty())
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);
    let body = body_to_string(res.into_body()).await;
    // Must return to where the user came from (/profile/offers/{id}), NOT fall back to /offers
    assert!(body.contains(&format!("href=\"/profile/offers/{}\"", offer.id)));

    // When clicking back from public view to owner management view, referer is the public view.
    // The owner view must resolve its back button to /profile/offers, NOT loop back to /offers/{slug}!
    let public_page_url = format!("http://localhost:3000/offers/{}", offer.slug);
    let req_owner = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("host", "localhost:3000")
        .header("cookie", format!("sid={session}"))
        .header("referer", &public_page_url)
        .body(Body::empty())
        .unwrap();

    let res_owner = router.handle(req_owner).await;
    assert_eq!(res_owner.status(), http::StatusCode::OK);
    let body_owner = body_to_string(res_owner.into_body()).await;
    let back_btn_expected = "href=\"/profile/offers\" class=\"child-back-btn\"";
    assert!(body_owner.contains(back_btn_expected));
    let back_btn_loop = format!("href=\"/offers/{}\" class=\"child-back-btn\"", offer.slug);
    assert!(!body_owner.contains(&back_btn_loop));
}
