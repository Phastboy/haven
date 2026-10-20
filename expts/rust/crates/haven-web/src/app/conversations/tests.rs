//! Tests for conversations, messages, inquiries, and unread notifications.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::similar_names,
    clippy::unimplemented,
    clippy::items_after_statements,
    reason = "Test assertions, indexing, and test fakes"
)]

use haven_domain::account::AccountId;
use haven_domain::conversation::{ConversationPair, MessageBody, NewMessage};
use haven_domain::fakes::FakeConversationRepository;
use haven_domain::offer::{CurrencyCode, Offer, OfferId, Price};
use haven_domain::ports::{
    AccountRepository, ConversationRepository, MagicLinkRepository, OfferImageRepository,
    OfferRepository, Registry, RepoError, SessionRepository, UserRepository,
};
use haven_domain::session::HashedToken;
use haven_domain::user::{DisplayName, ProfileHandle, User, UserId, UserProfileUpdate};
use http::Request;
use std::collections::HashMap;
use std::sync::Arc;
use topcoat::router::Body;
use topcoat::session::Token;

#[derive(Clone, Default)]
struct TestUserRepository {
    users_by_token: Arc<std::sync::Mutex<HashMap<String, User>>>,
    users: Arc<std::sync::Mutex<Vec<User>>>,
}

#[haven_domain::ports::async_trait]
impl UserRepository for TestUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepoError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.id == id).cloned())
    }
    async fn find_by_account(&self, _account_id: AccountId) -> Result<Option<User>, RepoError> {
        unimplemented!()
    }
    async fn find_by_handle(&self, _handle: &ProfileHandle) -> Result<Option<User>, RepoError> {
        unimplemented!()
    }
    async fn find_by_session(&self, token_hash: &HashedToken) -> Result<Option<User>, RepoError> {
        let lock = self.users_by_token.lock().unwrap();
        Ok(lock.get(token_hash.as_str()).cloned())
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
        _user_id: UserId,
        _profile: &UserProfileUpdate,
    ) -> Result<User, RepoError> {
        unimplemented!()
    }
    async fn update_avatar(
        &self,
        _user_id: UserId,
        _avatar: Option<&haven_domain::image::ImagePublicId>,
    ) -> Result<User, RepoError> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct TestRegistry {
    offers: haven_domain::fakes::FakeOfferRepository,
    users: TestUserRepository,
    conversations: FakeConversationRepository,
}

impl Registry for TestRegistry {
    fn offers(&self) -> &(dyn OfferRepository + 'static) {
        &self.offers
    }
    fn offer_images(&self) -> &(dyn OfferImageRepository + 'static) {
        &self.offers
    }
    fn accounts(&self) -> &(dyn AccountRepository + 'static) {
        unimplemented!()
    }
    fn users(&self) -> &(dyn UserRepository + 'static) {
        &self.users
    }
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static) {
        unimplemented!()
    }
    fn sessions(&self) -> &(dyn SessionRepository + 'static) {
        unimplemented!()
    }
    fn conversations(&self) -> &(dyn ConversationRepository + 'static) {
        &self.conversations
    }
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

fn setup_conversation_app() -> (
    topcoat::router::Router,
    TestRegistry,
    User,
    User,
    Offer,
    String,
    String,
) {
    let u1 = User {
        id: UserId::generate(),
        account_id: AccountId::generate(),
        display_name: DisplayName::parse("Maker User").unwrap(),
        handle: ProfileHandle::parse("maker").unwrap(),
        avatar_public_id: None,
        bio: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let u2 = User {
        id: UserId::generate(),
        account_id: AccountId::generate(),
        display_name: DisplayName::parse("Asker User").unwrap(),
        handle: ProfileHandle::parse("asker").unwrap(),
        avatar_public_id: None,
        bio: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let users_repo = TestUserRepository::default();
    users_repo.users.lock().unwrap().push(u1.clone());
    users_repo.users.lock().unwrap().push(u2.clone());

    let token1 = create_auth_session(&users_repo, u1.clone());
    let token2 = create_auth_session(&users_repo, u2.clone());

    let offers_repo = haven_domain::fakes::FakeOfferRepository::default();
    let offer = Offer {
        id: OfferId::generate(),
        user_id: u1.id,
        slug: haven_domain::offer::OfferSlug::parse("handmade-table-123456").unwrap(),
        title: "Handmade Table".to_string(),
        description: "A solid wooden dining table made with oak.".to_string(),
        price: Price::new(150).unwrap(),
        currency: CurrencyCode::parse("USD").unwrap(),
        image_count: 1,
        images: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    offers_repo.offers.lock().unwrap().push(offer.clone());

    let convs_repo = FakeConversationRepository::default();
    convs_repo
        .users_info
        .lock()
        .unwrap()
        .insert(u1.id, (u1.display_name.clone(), u1.handle.clone(), None));
    convs_repo
        .users_info
        .lock()
        .unwrap()
        .insert(u2.id, (u2.display_name.clone(), u2.handle.clone(), None));

    let registry = TestRegistry {
        offers: offers_repo,
        users: users_repo,
        conversations: convs_repo,
    };

    let state = crate::app::state::AppState {
        registry: Arc::new(registry.clone()),
        google_oauth: None,
        http_client: reqwest::Client::new(),
        email_delivery_enabled: Some(false),
        image_storage: Arc::new(crate::storage::MockImageStorage::new()),
        cloudinary_cloud_name: None,
        broadcaster: Arc::new(crate::app::broadcaster::InProcessBroadcaster::new()),
    };

    use topcoat::{
        cookie::RouterBuilderCookieExt, router::RouterBuilderDiscoverExt,
        session::RouterBuilderSessionExt,
    };
    let router = crate::app::router()
        .cookies()
        .sessions(
            topcoat::session::SessionConfig::builder()
                .token_store(crate::token_store::AdaptiveCookieTokenStore::new().name("sid"))
                .build(),
        )
        .app_context(state)
        .discover()
        .build();

    (router, registry, u1, u2, offer, token1, token2)
}

#[tokio::test]
async fn test_inbox_and_thread_flow() {
    let (router, reg, u1, u2, _offer, token1, _token2) = setup_conversation_app();

    // 1. Initially inbox is empty
    let req = Request::get("/conversations")
        .header("Cookie", format!("sid={token1}"))
        .body(Body::empty())
        .unwrap();
    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    // 2. Pair starts a conversation
    let pair = ConversationPair::new(u1.id, u2.id).unwrap();
    let conv = reg.conversations.get_or_create(pair).await.unwrap();
    reg.conversations
        .append(
            conv.id,
            NewMessage {
                sender_id: u2.id,
                body: MessageBody::parse("Is this table still available?").unwrap(),
                offer_id: None,
            },
        )
        .await
        .unwrap();

    // 3. Inbox now shows conversation
    let req2 = Request::get("/conversations")
        .header("Cookie", format!("sid={token1}"))
        .body(Body::empty())
        .unwrap();
    let res2 = router.handle(req2).await;
    assert_eq!(res2.status(), http::StatusCode::OK);

    // 4. View thread
    let req3 = Request::get(format!("/conversations/{}", conv.id))
        .header("Cookie", format!("sid={token1}"))
        .body(Body::empty())
        .unwrap();
    let res3 = router.handle(req3).await;
    assert_eq!(res3.status(), http::StatusCode::OK);

    // 5. Reply to thread
    let req4 = Request::post(format!("/conversations/{}/messages", conv.id))
        .header("Cookie", format!("sid={token1}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("body=Yes%2C+it+is+available!"))
        .unwrap();
    let res4 = router.handle(req4).await;
    assert_eq!(res4.status(), http::StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn test_offer_inquiry_flow() {
    let (router, reg, _u1, _u2, offer, token1, token2) = setup_conversation_app();

    // 1. Send inquiry from owner to own offer -> BAD_REQUEST (400)
    let req = Request::post(format!("/offers/{}/message", offer.slug.as_str()))
        .header("Cookie", format!("sid={token1}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("body=Hello+from+offer+page"))
        .unwrap();
    let res = router.handle(req).await;
    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    // 2. Send inquiry from asker -> SEE_OTHER (redirects to conversation)
    let req2 = Request::post(format!("/offers/{}/message", offer.slug.as_str()))
        .header("Cookie", format!("sid={token2}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("body=Hello+I+would+like+to+buy+this!"))
        .unwrap();
    let res2 = router.handle(req2).await;
    assert_eq!(res2.status(), http::StatusCode::SEE_OTHER);

    // Verify list_offer_inquirers reports this inquiry
    let inqs = reg
        .conversations
        .list_offer_inquirers(offer.id, None, 10)
        .await
        .unwrap();
    assert_eq!(inqs.items.len(), 1);
    assert_eq!(inqs.items[0].message_count, 1);
}
