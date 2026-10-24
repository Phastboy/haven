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

#[allow(clippy::struct_field_names, reason = "Test registry mock fields")]
pub struct TestRegistry {
    pub offers_repo: FakeOfferRepository,
    pub users_repo: TestUserRepository,
    pub convs_repo: haven_domain::fakes::FakeConversationRepository,
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
    fn conversations(&self) -> &(dyn haven_domain::ports::ConversationRepository + 'static) {
        &self.convs_repo
    }
    fn saved_offers(&self) -> &(dyn haven_domain::ports::SavedOfferRepository + 'static) {
        unimplemented!()
    }
    fn follows(&self) -> &(dyn haven_domain::ports::FollowRepository + 'static) {
        unimplemented!()
    }
}

fn setup_test_app() -> (topcoat::router::Router, TestRegistry) {
    let registry = TestRegistry {
        offers_repo: FakeOfferRepository::default(),
        users_repo: TestUserRepository::default(),
        convs_repo: haven_domain::fakes::FakeConversationRepository::default(),
    };

    let state = crate::app::state::AppState {
        registry: Arc::new(TestRegistry {
            offers_repo: registry.offers_repo.clone(),
            users_repo: registry.users_repo.clone(),
            convs_repo: registry.convs_repo.clone(),
        }),
        google_oauth: None,
        http_client: reqwest::Client::new(),
        email_delivery_enabled: Some(false),
        image_storage: Arc::new(crate::storage::MockImageStorage::new()),
        cloudinary_cloud_name: None,
        broadcaster: Arc::new(crate::app::broadcaster::InProcessBroadcaster::new()),
    };

    let origins = crate::app::origin::compute_trusted_origins("3000", None, None);
    let origin_policy = topcoat::router::OriginPolicy::new().trust_origins(origins);
    let router = crate::app::router()
        .origin_policy(origin_policy)
        .layer(topcoat::router::BodyLimit::max(6 * 1024 * 1024))
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
#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive integration test for slug caching and headers"
)]
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
        .unwrap()
        .into_inner();

    let pub_id = haven_domain::image::ImagePublicId::parse("offers/test/cover.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    registry
        .offers_repo
        .add_image(offer.id, user_id, &pub_id, dims)
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
        .unwrap()
        .into_inner();

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
        .unwrap()
        .into_inner();

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
        .unwrap()
        .into_inner();

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

    // 3. Known user with an offer without images remains hidden on public profile
    let create_offer = CreateOffer {
        title: "Handmade Wooden Chair".to_string(),
        description: "Solid oak handcrafted dining chair.".to_string(),
        price: Price::new(15000).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let created_status = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap();
    let created = created_status.into_inner();

    // Without images, public profile still shows "No offers yet"
    let req_no_img = Request::builder()
        .method("GET")
        .uri("/profiles/alice-maker")
        .body(Body::empty())
        .unwrap();
    let res_no_img = router.handle(req_no_img).await;
    assert_eq!(res_no_img.status(), http::StatusCode::OK);
    let body_no_img = body_to_string(res_no_img.into_body()).await;
    assert!(body_no_img.contains("No offers yet"));

    // Add an image: now the offer becomes publicly visible
    let pub_id = haven_domain::image::ImagePublicId::parse("offers/alice/chair.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    registry
        .offers_repo
        .add_image(created.id, user.id, &pub_id, dims)
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
    assert!(body_get.contains("href=\"/profile/edit\""));
    assert!(body_get.contains("My offers"));
    assert!(body_get.contains("href=\"/profile/offers\""));
    assert!(body_get.contains("Sign out"));
    assert!(body_get.contains("id=\"signout-dialog\""));
    // Header has clean nav
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
        .unwrap()
        .into_inner();

    let pub_id = haven_domain::image::ImagePublicId::parse("offers/bob/vase.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    registry
        .offers_repo
        .add_image(offer.id, user.id, &pub_id, dims)
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
#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive integration test for navigation layout hierarchy"
)]
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
        .unwrap()
        .into_inner();

    let pub_id = haven_domain::image::ImagePublicId::parse("offers/nav/bowl.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    registry
        .offers_repo
        .add_image(offer.id, user.id, &pub_id, dims)
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
    assert!(body_main.contains("bottom-nav"));

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
    assert!(!body_child_no_ref.contains("bottom-nav"));

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
        .unwrap()
        .into_inner();

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

#[allow(
    clippy::too_many_arguments,
    reason = "Test helper builds multipart body from field name, filename, content type, and bytes"
)]
fn create_multipart_image(
    field_name: &str,
    filename: &str,
    content_type: &str,
    bytes: &[u8],
) -> (String, Vec<u8>) {
    let boundary = "------------------------testboundary1234567890";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    (format!("multipart/form-data; boundary={boundary}"), body)
}

#[tokio::test]
async fn test_avatar_upload_success_and_views_rendering() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "avatar-maker");
    let session = create_auth_session(&registry.users_repo, user.clone());

    // Valid JPEG magic bytes header: FF D8 FF E0 ...
    let valid_jpeg_bytes = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB,
    ];

    let (content_type, body_bytes) =
        create_multipart_image("avatar", "profile.jpg", "image/jpeg", &valid_jpeg_bytes);

    // 1. Authenticated POST /profile/avatar with valid JPEG succeeds and redirects to /profile/edit?avatar_updated=1
    let req_upload = Request::builder()
        .method("POST")
        .uri("/profile/avatar")
        .header("cookie", format!("sid={session}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", content_type)
        .body(Body::from(body_bytes))
        .unwrap();

    let res_upload = router.handle(req_upload).await;
    assert_eq!(res_upload.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res_upload
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/profile/edit?avatar_updated=1"
    );

    // Verify user in repository now has avatar_public_id set
    let updated_user = registry
        .users_repo
        .find_by_id(user.id)
        .await
        .unwrap()
        .unwrap();
    assert!(updated_user.avatar_public_id.is_some());
    let public_id_str = updated_user.avatar_public_id.as_ref().unwrap().as_str();

    // 2. GET /profile renders avatar image with srcset
    let req_profile = Request::builder()
        .method("GET")
        .uri("/profile")
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();

    let res_profile = router.handle(req_profile).await;
    assert_eq!(res_profile.status(), http::StatusCode::OK);
    let body_profile = body_to_string(res_profile.into_body()).await;
    assert!(body_profile.contains(&format!(
        "https://res.cloudinary.com/haven/image/private/t_avatar/{public_id_str}"
    )));
    assert!(body_profile.contains("srcset="));
    assert!(body_profile.contains(&format!(
        "https://res.cloudinary.com/haven/image/private/t_avatar_2x/{public_id_str} 2x"
    )));

    // 3. GET /profiles/{handle} (public view) renders avatar image with srcset
    let req_public = Request::builder()
        .method("GET")
        .uri(format!("/profiles/{}", user.handle.as_str()))
        .body(Body::empty())
        .unwrap();

    let res_public = router.handle(req_public).await;
    assert_eq!(res_public.status(), http::StatusCode::OK);
    let body_public = body_to_string(res_public.into_body()).await;
    assert!(body_public.contains(&format!(
        "https://res.cloudinary.com/haven/image/private/t_avatar/{public_id_str}"
    )));

    // 4. GET /profile/edit?avatar_updated=1 renders success notice and "Remove photo" button
    let req_edit = Request::builder()
        .method("GET")
        .uri("/profile/edit?avatar_updated=1")
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();

    let res_edit = router.handle(req_edit).await;
    assert_eq!(res_edit.status(), http::StatusCode::OK);
    let body_edit = body_to_string(res_edit.into_body()).await;
    assert!(body_edit.contains("Avatar photo updated successfully."));
    assert!(body_edit.contains("action=\"/profile/avatar/delete\""));
    assert!(body_edit.contains("Remove photo"));
}

#[tokio::test]
async fn test_avatar_upload_rejections() {
    let (router, registry) = setup_test_app();

    // 1. Unauthenticated POST /profile/avatar redirects to /auth/sign-in
    let req_unauth = Request::builder()
        .method("POST")
        .uri("/profile/avatar")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap();

    let res_unauth = router.handle(req_unauth).await;
    assert!(res_unauth.status().is_redirection());
    assert_eq!(
        res_unauth
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in"
    );

    let user = mock_user(UserId::generate(), AccountId::generate(), "reject-tester");
    let session = create_auth_session(&registry.users_repo, user.clone());

    // 2. Reject SVG upload (disallowed format / script injection prevention)
    let svg_bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    let (content_type_svg, body_svg) =
        create_multipart_image("avatar", "vector.svg", "image/svg+xml", svg_bytes);

    let req_svg = Request::builder()
        .method("POST")
        .uri("/profile/avatar")
        .header("cookie", format!("sid={session}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", content_type_svg)
        .body(Body::from(body_svg))
        .unwrap();

    let res_svg = router.handle(req_svg).await;
    assert_eq!(res_svg.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res_svg.headers().get("location").unwrap().to_str().unwrap(),
        "/profile/edit?error=invalid_format"
    );

    // 3. Reject empty upload
    let (content_type_empty, body_empty) =
        create_multipart_image("avatar", "empty.jpg", "image/jpeg", &[]);

    let req_empty = Request::builder()
        .method("POST")
        .uri("/profile/avatar")
        .header("cookie", format!("sid={session}"))
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", content_type_empty)
        .body(Body::from(body_empty))
        .unwrap();

    let res_empty = router.handle(req_empty).await;
    assert_eq!(res_empty.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        res_empty
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/profile/edit?error=empty_file"
    );

    // User's avatar must still be None
    let user_check = registry
        .users_repo
        .find_by_id(user.id)
        .await
        .unwrap()
        .unwrap();
    assert!(user_check.avatar_public_id.is_none());
}

#[tokio::test]
async fn test_avatar_deletion() {
    let (router, registry) = setup_test_app();

    // 1. Unauthenticated POST /profile/avatar/delete redirects to sign-in
    let req_unauth = Request::builder()
        .method("POST")
        .uri("/profile/avatar/delete")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap();

    let res_unauth = router.handle(req_unauth).await;
    assert!(res_unauth.status().is_redirection());
    assert_eq!(
        res_unauth
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "/auth/sign-in"
    );

    // 2. Authenticated user with avatar deletes avatar
    let mut user = mock_user(UserId::generate(), AccountId::generate(), "delete-tester");
    let public_id = haven_domain::image::ImagePublicId::parse("users/123/existing-avatar").unwrap();
    user.avatar_public_id = Some(public_id);
    let session = create_auth_session(&registry.users_repo, user.clone());

    let req_delete = Request::builder()
        .method("POST")
        .uri("/profile/avatar/delete")
        .header("cookie", format!("sid={session}"))
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
        "/profile/edit?avatar_deleted=1"
    );

    // User avatar must now be None
    let user_check = registry
        .users_repo
        .find_by_id(user.id)
        .await
        .unwrap()
        .unwrap();
    assert!(user_check.avatar_public_id.is_none());

    // GET /profile/edit?avatar_deleted=1 shows removed notice and no remove button
    let req_edit = Request::builder()
        .method("GET")
        .uri("/profile/edit?avatar_deleted=1")
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();

    let res_edit = router.handle(req_edit).await;
    assert_eq!(res_edit.status(), http::StatusCode::OK);
    let body_edit = body_to_string(res_edit.into_body()).await;
    assert!(body_edit.contains("Avatar photo removed."));
    assert!(!body_edit.contains("action=\"/profile/avatar/delete\""));
}

fn create_multipart_form(
    fields: &[(&str, &str)],
    files: &[(&str, &str, &str, &[u8])],
) -> (String, Vec<u8>) {
    let boundary = "------------------------testboundary9876543210";
    let mut body = Vec::new();
    for (name, val) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(val.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    for (name, filename, content_type, bytes) in files {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

#[tokio::test]
async fn test_offer_visibility_rules_and_slug_404() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "vis-user");
    let session = create_auth_session(&registry.users_repo, user.clone());

    let other_user = mock_user(UserId::generate(), AccountId::generate(), "other-vis-user");
    let other_session = create_auth_session(&registry.users_repo, other_user);

    let create_offer = CreateOffer {
        title: "Invisible Without Photo".to_string(),
        description: "This offer should be hidden until a photo is uploaded.".to_string(),
        price: Price::new(100).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap()
        .into_inner();

    let slug_url = format!("/offers/{}", offer.slug);

    // 1. Unauthenticated request to /offers/{slug} returns 404
    let req_anon = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .body(Body::empty())
        .unwrap();
    let res_anon = router.handle(req_anon).await;
    assert_eq!(res_anon.status(), http::StatusCode::NOT_FOUND);

    // 2. Non-owner authenticated request to /offers/{slug} returns 404
    let req_other = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("cookie", format!("sid={other_session}"))
        .body(Body::empty())
        .unwrap();
    let res_other = router.handle(req_other).await;
    assert_eq!(res_other.status(), http::StatusCode::NOT_FOUND);

    // 3. Unauthenticated request to /offers feed does NOT contain this offer
    let req_feed = Request::builder()
        .method("GET")
        .uri("/offers")
        .body(Body::empty())
        .unwrap();
    let res_feed = router.handle(req_feed).await;
    assert_eq!(res_feed.status(), http::StatusCode::OK);
    let body_feed = body_to_string(res_feed.into_body()).await;
    assert!(!body_feed.contains("Invisible Without Photo"));

    // 4. Owner GET /profile/offers/{id} renders calm banner: "Only you can see this offer. Add a photo to make it public."
    let req_owner_view = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();
    let res_owner_view = router.handle(req_owner_view).await;
    assert_eq!(res_owner_view.status(), http::StatusCode::OK);
    let body_owner_view = body_to_string(res_owner_view.into_body()).await;
    assert!(
        body_owner_view.contains("Only you can see this offer. Add a photo to make it public.")
    );
    assert!(
        body_owner_view
            .contains("No photos yet. Add at least one photo to make this offer public.")
    );

    // 5. Owner GET /offers/{slug} succeeds (owner preview permitted)
    let req_owner_slug = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();
    let res_owner_slug = router.handle(req_owner_slug).await;
    assert_eq!(res_owner_slug.status(), http::StatusCode::OK);

    // 6. Add a photo to make the offer public
    let pub_id = haven_domain::image::ImagePublicId::parse("offers/vis/cover.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    registry
        .offers_repo
        .add_image(offer.id, user.id, &pub_id, dims)
        .await
        .unwrap();

    // 7. Unauthenticated request to /offers/{slug} now returns 200 and renders detail photo stack
    let req_anon_after = Request::builder()
        .method("GET")
        .uri(&slug_url)
        .body(Body::empty())
        .unwrap();
    let res_anon_after = router.handle(req_anon_after).await;
    assert_eq!(res_anon_after.status(), http::StatusCode::OK);
    let body_anon_after = body_to_string(res_anon_after.into_body()).await;
    assert!(body_anon_after.contains("offer-photo-stack"));
    assert!(body_anon_after.contains("Invisible Without Photo, photo 1 of 1"));
    assert!(body_anon_after.contains("t_detail"));

    // 8. Public feed /offers now includes the offer with cover image
    let req_feed_after = Request::builder()
        .method("GET")
        .uri("/offers")
        .body(Body::empty())
        .unwrap();
    let res_feed_after = router.handle(req_feed_after).await;
    assert_eq!(res_feed_after.status(), http::StatusCode::OK);
    let body_feed_after = body_to_string(res_feed_after.into_body()).await;
    assert!(body_feed_after.contains("Invisible Without Photo"));
    assert!(body_feed_after.contains("offer-card-image"));
    assert!(body_feed_after.contains("t_card"));
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive integration test for offer image uploads and limits"
)]
async fn test_offer_image_upload_and_limit() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "upload-tester");
    let session = create_auth_session(&registry.users_repo, user.clone());

    let create_offer = CreateOffer {
        title: "Photo Upload Showcase".to_string(),
        description: "Demonstrating image management and limits.".to_string(),
        price: Price::new(500).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap()
        .into_inner();

    let valid_jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB,
    ];

    // 1. Upload invalid format (e.g. text/SVG)
    let (bad_ct, bad_body) =
        create_multipart_image("image", "script.svg", "image/svg+xml", b"<svg></svg>");
    let req_bad = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", bad_ct)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(bad_body))
        .unwrap();
    let res_bad = router.handle(req_bad).await;
    assert_eq!(res_bad.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_bad
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("error=invalid_format")
    );

    // 2. Upload valid image 1
    let (good_ct, good_body) =
        create_multipart_image("image", "photo1.jpg", "image/jpeg", &valid_jpeg);
    let req_good1 = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", good_ct)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(good_body))
        .unwrap();
    let res_good1 = router.handle(req_good1).await;
    assert_eq!(res_good1.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_good1
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("image_added=1")
    );

    // Upload 4 more valid images to hit max 5
    for i in 2..=5 {
        let (ct, body) =
            create_multipart_image("image", &format!("photo{i}.jpg"), "image/jpeg", &valid_jpeg);
        let req = Request::builder()
            .method("POST")
            .uri(format!("/profile/offers/{}/images", offer.id))
            .header("cookie", format!("sid={session}"))
            .header("content-type", ct)
            .header("origin", "http://localhost:3000")
            .header("sec-fetch-site", "same-origin")
            .body(Body::from(body))
            .unwrap();
        let res = router.handle(req).await;
        assert_eq!(res.status(), http::StatusCode::SEE_OTHER);
    }

    // 3. GET /profile/offers/{id} at max limit renders "5 of 5 photos. Remove one to add another." and hides upload form
    let req_max_view = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();
    let res_max_view = router.handle(req_max_view).await;
    assert_eq!(res_max_view.status(), http::StatusCode::OK);
    let body_max_view = body_to_string(res_max_view.into_body()).await;
    assert!(body_max_view.contains("5 of 5 photos. Remove one to add another."));
    assert!(!body_max_view.contains("class=\"image-upload-form\""));
    assert!(!body_max_view.contains("type=\"file\""));

    // 4. Attempt 6th upload returns error=max_images_reached
    let (extra_ct, extra_body) =
        create_multipart_image("image", "photo6.jpg", "image/jpeg", &valid_jpeg);
    let req_extra = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", extra_ct)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(extra_body))
        .unwrap();
    let res_extra = router.handle(req_extra).await;
    assert_eq!(res_extra.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_extra
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("error=max_images_reached")
    );
}

#[tokio::test]
async fn test_offer_image_reordering() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "reorder-tester");
    let session = create_auth_session(&registry.users_repo, user.clone());

    let create_offer = CreateOffer {
        title: "Reorderable Photos".to_string(),
        description: "Testing image positions.".to_string(),
        price: Price::new(100).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap()
        .into_inner();

    let pub1 = haven_domain::image::ImagePublicId::parse("offers/reorder/first.jpg").unwrap();
    let pub2 = haven_domain::image::ImagePublicId::parse("offers/reorder/second.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();

    let img1 = registry
        .offers_repo
        .add_image(offer.id, user.id, &pub1, dims)
        .await
        .unwrap();
    let img2 = registry
        .offers_repo
        .add_image(offer.id, user.id, &pub2, dims)
        .await
        .unwrap();

    // Reorder: Move img2 up
    let form_body = format!("image_id={}&direction=up", img2.id.as_uuid());
    let req_reorder = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images/reorder", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(form_body))
        .unwrap();

    let res_reorder = router.handle(req_reorder).await;
    assert_eq!(res_reorder.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_reorder
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("reordered=1")
    );

    // Verify in repo that img2 is now position 1 (cover) and img1 is position 2
    let updated_images = registry.offers_repo.find_by_offer(offer.id).await.unwrap();
    assert_eq!(updated_images.len(), 2);
    assert_eq!(updated_images[0].id, img2.id);
    assert_eq!(updated_images[0].position.as_u8(), 1);
    assert_eq!(updated_images[1].id, img1.id);
    assert_eq!(updated_images[1].position.as_u8(), 2);
}

#[tokio::test]
async fn test_offer_image_deletion_and_protection() {
    let (router, registry) = setup_test_app();

    let user = mock_user(
        UserId::generate(),
        AccountId::generate(),
        "delete-tester-offer",
    );
    let session = create_auth_session(&registry.users_repo, user.clone());

    let create_offer = CreateOffer {
        title: "Delete Protection Offer".to_string(),
        description: "Checking last image protection.".to_string(),
        price: Price::new(200).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap()
        .into_inner();

    let pub1 = haven_domain::image::ImagePublicId::parse("offers/protect/only.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    let img1 = registry
        .offers_repo
        .add_image(offer.id, user.id, &pub1, dims)
        .await
        .unwrap();

    // 1. Attempt to delete the only image -> rejected with ?error=last_image
    let form_delete_last = format!("image_id={}", img1.id.as_uuid());
    let req_del_last = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images/delete", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(form_delete_last))
        .unwrap();
    let res_del_last = router.handle(req_del_last).await;
    assert_eq!(res_del_last.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_del_last
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("error=last_image")
    );

    // 2. GET /profile/offers/{id} renders protection text
    let req_view = Request::builder()
        .method("GET")
        .uri(format!("/profile/offers/{}", offer.id))
        .header("cookie", format!("sid={session}"))
        .body(Body::empty())
        .unwrap();
    let res_view = router.handle(req_view).await;
    assert_eq!(res_view.status(), http::StatusCode::OK);
    let body_view = body_to_string(res_view.into_body()).await;
    assert!(
        body_view
            .contains("An offer needs at least one photo. Add another first, or replace this one.")
    );

    // 3. Add a second image
    let pub2 = haven_domain::image::ImagePublicId::parse("offers/protect/second.jpg").unwrap();
    let img2 = registry
        .offers_repo
        .add_image(offer.id, user.id, &pub2, dims)
        .await
        .unwrap();

    // 4. Delete the second image -> succeeds!
    let form_delete_second = format!("image_id={}", img2.id.as_uuid());
    let req_del_second = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images/delete", offer.id))
        .header("cookie", format!("sid={session}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(form_delete_second))
        .unwrap();
    let res_del_second = router.handle(req_del_second).await;
    assert_eq!(res_del_second.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_del_second
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("image_deleted=1")
    );

    // Image 2 is now deleted from repo
    let remaining = registry.offers_repo.find_by_offer(offer.id).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, img1.id);
}

#[tokio::test]
async fn test_offer_create_with_multipart_photo() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "creator-user");
    let session = create_auth_session(&registry.users_repo, user.clone());

    let valid_jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB,
    ];

    let (ct, body) = create_multipart_form(
        &[
            ("title", "Multipart Handmade Quilt"),
            ("description", "Cozy patchwork quilt handmade from cotton."),
            ("price", "120"),
            ("currency", "USD"),
        ],
        &[("photo", "quilt.jpg", "image/jpeg", &valid_jpeg)],
    );

    let req = Request::builder()
        .method("POST")
        .uri("/profile/offers/new")
        .header("cookie", format!("sid={session}"))
        .header("content-type", ct)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(body))
        .unwrap();

    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::SEE_OTHER);
    let location = res.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.starts_with("/profile/offers/"));

    // Verify offer created in repo has image attached and is public
    let user_offers = registry.offers_repo.find_by_user(user.id).await.unwrap();
    assert_eq!(user_offers.len(), 1);
    let created = &user_offers[0];
    assert_eq!(created.title, "Multipart Handmade Quilt");
    assert_eq!(created.image_count, 1);
    assert!(created.is_public());
    assert!(created.cover_image().is_some());
}

#[tokio::test]
#[allow(
    clippy::too_many_lines,
    reason = "Comprehensive security test for IDOR and upload boundaries"
)]
async fn test_offer_images_security_and_authorization() {
    let (router, registry) = setup_test_app();

    let user1 = mock_user(UserId::generate(), AccountId::generate(), "owner-user");
    let session1 = create_auth_session(&registry.users_repo, user1.clone());

    let user2 = mock_user(UserId::generate(), AccountId::generate(), "attacker-user");
    let session2 = create_auth_session(&registry.users_repo, user2);

    let create_offer = CreateOffer {
        title: "Secure Offer".to_string(),
        description: "Protected against IDOR and CSRF.".to_string(),
        price: Price::new(250).unwrap(),
        currency: CurrencyCode::default_code(),
    };
    let offer = registry
        .offers_repo
        .create(user1.id, IdempotencyKey::new(Uuid::new_v4()), &create_offer)
        .await
        .unwrap()
        .into_inner();

    let pub1 = haven_domain::image::ImagePublicId::parse("offers/sec/img1.jpg").unwrap();
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    let img1 = registry
        .offers_repo
        .add_image(offer.id, user1.id, &pub1, dims)
        .await
        .unwrap();

    let valid_jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01, 0x00,
        0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB,
    ];

    // 1. IDOR: User 2 cannot upload images to User 1's offer (returns 404)
    let (ct2, body2) = create_multipart_image("image", "hack.jpg", "image/jpeg", &valid_jpeg);
    let req_idor_upload = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session2}"))
        .header("content-type", ct2)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(body2))
        .unwrap();
    let res_idor_upload = router.handle(req_idor_upload).await;
    assert_eq!(res_idor_upload.status(), http::StatusCode::NOT_FOUND);

    // 2. IDOR: User 2 cannot delete images from User 1's offer (returns 404)
    let req_idor_delete = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images/delete", offer.id))
        .header("cookie", format!("sid={session2}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(format!("image_id={}", img1.id.as_uuid())))
        .unwrap();
    let res_idor_delete = router.handle(req_idor_delete).await;
    assert_eq!(res_idor_delete.status(), http::StatusCode::NOT_FOUND);

    // 3. IDOR: User 2 cannot reorder images on User 1's offer (returns 404)
    let req_idor_reorder = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images/reorder", offer.id))
        .header("cookie", format!("sid={session2}"))
        .header("content-type", "application/x-www-form-urlencoded")
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(format!(
            "image_id={}&direction=up",
            img1.id.as_uuid()
        )))
        .unwrap();
    let res_idor_reorder = router.handle(req_idor_reorder).await;
    assert_eq!(res_idor_reorder.status(), http::StatusCode::NOT_FOUND);

    // 4. File size cap: User 1 uploading oversized image (> 5 MB) is rejected with ?error=file_too_large
    let oversized = vec![0xFF; haven_domain::image::MAX_IMAGE_BYTES + 1];
    let (large_ct, large_body) =
        create_multipart_image("image", "huge.jpg", "image/jpeg", &oversized);
    let req_large = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session1}"))
        .header("content-type", large_ct)
        .header("origin", "http://localhost:3000")
        .header("sec-fetch-site", "same-origin")
        .body(Body::from(large_body))
        .unwrap();
    let res_large = router.handle(req_large).await;
    assert_eq!(res_large.status(), http::StatusCode::SEE_OTHER);
    assert!(
        res_large
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("error=too_large")
    );

    // 5. CSRF defense: Cross-origin POST from untrusted origin is rejected with 403 Forbidden
    let (ct_csrf, body_csrf) =
        create_multipart_image("image", "csrf.jpg", "image/jpeg", &valid_jpeg);
    let req_csrf = Request::builder()
        .method("POST")
        .uri(format!("/profile/offers/{}/images", offer.id))
        .header("cookie", format!("sid={session1}"))
        .header("content-type", ct_csrf)
        .header("origin", "https://malicious-site.example.com")
        .header("sec-fetch-site", "cross-site")
        .body(Body::from(body_csrf))
        .unwrap();
    let res_csrf = router.handle(req_csrf).await;
    assert_eq!(res_csrf.status(), http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_feed_search_and_price_filtering_and_empty_states() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "artisan-search");
    let _ = create_auth_session(&registry.users_repo, user.clone());

    // Create 3 offers:
    // 1. Handmade Oak Desk - NGN 5,000
    // 2. Vintage Glass Table - NGN 15,000
    // 3. Free Wooden Chair - NGN 0 (Free)
    // 4. US Designer Lamp - USD 5,000
    let o1 = registry
        .offers_repo
        .create(
            user.id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Handmade Oak Desk".to_string(),
                description: "Sturdy solid oak desk handcrafted with love.".to_string(),
                price: Price::new(5000).unwrap(),
                currency: CurrencyCode::Ngn,
            },
        )
        .await
        .unwrap()
        .into_inner();

    let o2 = registry
        .offers_repo
        .create(
            user.id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Vintage Glass Table".to_string(),
                description: "Mid-century round glass coffee table.".to_string(),
                price: Price::new(15000).unwrap(),
                currency: CurrencyCode::Ngn,
            },
        )
        .await
        .unwrap()
        .into_inner();

    let o3 = registry
        .offers_repo
        .create(
            user.id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "Free Wooden Chair".to_string(),
                description: "Good wooden chair ready for porch or patio.".to_string(),
                price: Price::ZERO,
                currency: CurrencyCode::Ngn,
            },
        )
        .await
        .unwrap()
        .into_inner();

    let o4 = registry
        .offers_repo
        .create(
            user.id,
            IdempotencyKey::new(Uuid::new_v4()),
            &CreateOffer {
                title: "US Designer Lamp".to_string(),
                description: "Modern brass desk lamp priced in USD.".to_string(),
                price: Price::new(5000).unwrap(),
                currency: CurrencyCode::Usd,
            },
        )
        .await
        .unwrap()
        .into_inner();

    // Attach cover images so all offers are public
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    for off in [&o1, &o2, &o3, &o4] {
        let pub_id = haven_domain::image::ImagePublicId::parse(format!(
            "offers/search/{}",
            Uuid::new_v4().simple()
        ))
        .unwrap();
        registry
            .offers_repo
            .add_image(off.id, user.id, &pub_id, dims)
            .await
            .unwrap();
    }

    // 1. Unfiltered feed shows all offers
    let req_all = Request::builder()
        .method("GET")
        .uri("/offers")
        .body(Body::empty())
        .unwrap();
    let res_all = router.handle(req_all).await;
    assert_eq!(res_all.status(), http::StatusCode::OK);
    let body_all = body_to_string(res_all.into_body()).await;
    assert!(body_all.contains("Handmade Oak Desk"));
    assert!(body_all.contains("Vintage Glass Table"));
    assert!(body_all.contains("Free Wooden Chair"));
    assert!(body_all.contains("US Designer Lamp"));

    // 2. Title search for "Oak" returns only Oak Desk
    let req_oak = Request::builder()
        .method("GET")
        .uri("/offers?q=Oak")
        .body(Body::empty())
        .unwrap();
    let res_oak = router.handle(req_oak).await;
    assert_eq!(res_oak.status(), http::StatusCode::OK);
    let body_oak = body_to_string(res_oak.into_body()).await;
    assert!(body_oak.contains("Handmade Oak Desk"));
    assert!(!body_oak.contains("Vintage Glass Table"));
    assert!(!body_oak.contains("US Designer Lamp"));
    assert!(body_oak.contains("value=\"Oak\""));

    // 3. Price filter: NGN 1,000 to 10,000 returns only Oak Desk
    let req_price = Request::builder()
        .method("GET")
        .uri("/offers?currency=NGN&min_price=1000&max_price=10000")
        .body(Body::empty())
        .unwrap();
    let res_price = router.handle(req_price).await;
    assert_eq!(res_price.status(), http::StatusCode::OK);
    let body_price = body_to_string(res_price.into_body()).await;
    assert!(body_price.contains("Handmade Oak Desk"));
    assert!(!body_price.contains("Vintage Glass Table")); // 15000 > 10000
    assert!(!body_price.contains("Free Wooden Chair")); // 0 < 1000
    assert!(!body_price.contains("US Designer Lamp")); // Currency is USD

    // 4. Free filter: NGN with max_price=0 returns only Free Wooden Chair
    let req_free = Request::builder()
        .method("GET")
        .uri("/offers?currency=NGN&max_price=0")
        .body(Body::empty())
        .unwrap();
    let res_free = router.handle(req_free).await;
    assert_eq!(res_free.status(), http::StatusCode::OK);
    let body_free = body_to_string(res_free.into_body()).await;
    assert!(body_free.contains("Free Wooden Chair"));
    assert!(!body_free.contains("Handmade Oak Desk"));
    assert!(!body_free.contains("Vintage Glass Table"));

    // 5. Validation error: price filter without currency
    let req_no_curr = Request::builder()
        .method("GET")
        .uri("/offers?min_price=1000")
        .body(Body::empty())
        .unwrap();
    let res_no_curr = router.handle(req_no_curr).await;
    assert_eq!(res_no_curr.status(), http::StatusCode::OK);
    let body_no_curr = body_to_string(res_no_curr.into_body()).await;
    assert!(body_no_curr.contains("Currency is required when filtering by price."));

    // 6. Empty state: no matching offers
    let req_nomatch = Request::builder()
        .method("GET")
        .uri("/offers?q=NonExistentItem999")
        .body(Body::empty())
        .unwrap();
    let res_nomatch = router.handle(req_nomatch).await;
    assert_eq!(res_nomatch.status(), http::StatusCode::OK);
    let body_nomatch = body_to_string(res_nomatch.into_body()).await;
    assert!(body_nomatch.contains("No offers match your filters"));
    assert!(body_nomatch.contains("href=\"/offers\""));
    assert!(body_nomatch.contains("Clear all filters"));
}

#[tokio::test]
async fn test_feed_pagination_survives_filter_params() {
    let (router, registry) = setup_test_app();

    let user = mock_user(UserId::generate(), AccountId::generate(), "artisan-pages");
    let _ = create_auth_session(&registry.users_repo, user.clone());

    // Create 25 matching offers (feed page size is 20)
    let dims = haven_domain::image::ImageDimensions::new(640, 480).unwrap();
    for i in 1..=25 {
        let offer = registry
            .offers_repo
            .create(
                user.id,
                IdempotencyKey::new(Uuid::new_v4()),
                &CreateOffer {
                    title: format!("Filterable Ceramic Pot #{i:02}"),
                    description: "Hand-thrown stoneware ceramic plant pot.".to_string(),
                    price: Price::new(2500).unwrap(),
                    currency: CurrencyCode::Ngn,
                },
            )
            .await
            .unwrap()
            .into_inner();

        let pub_id = haven_domain::image::ImagePublicId::parse(format!(
            "offers/pot/{}",
            Uuid::new_v4().simple()
        ))
        .unwrap();
        registry
            .offers_repo
            .add_image(offer.id, user.id, &pub_id, dims)
            .await
            .unwrap();
    }

    // Request page 1 with search and price filters
    let req_p1 = Request::builder()
        .method("GET")
        .uri("/offers?q=Ceramic&currency=NGN&min_price=2000&max_price=3000")
        .body(Body::empty())
        .unwrap();
    let res_p1 = router.handle(req_p1).await;
    assert_eq!(res_p1.status(), http::StatusCode::OK);
    let body_p1 = body_to_string(res_p1.into_body()).await;

    // Check that the pagination next link contains all filter params preserved
    assert!(body_p1.contains("q=Ceramic"));
    assert!(body_p1.contains("currency=NGN"));
    assert!(body_p1.contains("min_price=2000"));
    assert!(body_p1.contains("max_price=3000"));
    assert!(body_p1.contains("after="));
}
