//! HTTP regression tests for independent unread loading.

use super::*;
use haven_domain::conversation::{
    Conversation, ConversationId, ConversationInboxPage, Message, MessagePage, OfferInquirersPage,
};
use tokio::sync::Notify;
use topcoat::router::{response::Response, to_bytes};

#[derive(Default)]
struct ControlledUnread {
    entered: Notify,
    release: Notify,
    fail: bool,
    people: std::sync::Mutex<Vec<UserId>>,
}

#[haven_domain::ports::async_trait]
impl ConversationRepository for ControlledUnread {
    async fn get_or_create(&self, _pair: ConversationPair) -> Result<Conversation, RepoError> {
        unimplemented!()
    }
    async fn find_by_id(&self, _id: ConversationId) -> Result<Option<Conversation>, RepoError> {
        unimplemented!()
    }
    async fn append(
        &self,
        _id: ConversationId,
        _message: NewMessage,
    ) -> Result<Message, RepoError> {
        unimplemented!()
    }
    async fn list_for_user(
        &self,
        _person: UserId,
        _cursor: Option<&str>,
        _limit: usize,
    ) -> Result<ConversationInboxPage, RepoError> {
        unimplemented!()
    }
    async fn messages(
        &self,
        _id: ConversationId,
        _cursor: Option<&str>,
        _limit: usize,
    ) -> Result<MessagePage, RepoError> {
        unimplemented!()
    }
    async fn mark_read(&self, _id: ConversationId, _person: UserId) -> Result<(), RepoError> {
        unimplemented!()
    }
    async fn unread_exists(&self, person: UserId) -> Result<bool, RepoError> {
        self.people.lock().unwrap().push(person);
        self.entered.notify_one();
        if self.fail {
            return Err(RepoError::Unavailable("unread storage unavailable".into()));
        }
        self.release.notified().await;
        Ok(true)
    }
    async fn list_offer_inquirers(
        &self,
        _id: OfferId,
        _cursor: Option<&str>,
        _limit: usize,
    ) -> Result<OfferInquirersPage, RepoError> {
        unimplemented!()
    }
}

fn request(path: &str, token: &str) -> http::Request<Body> {
    Request::get(path)
        .header("Cookie", format!("sid={token}"))
        .body(Body::empty())
        .unwrap()
}

fn assert_private(response: &Response) {
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(response.headers()["vary"], "Cookie");
    assert_eq!(response.headers()["x-robots-tag"], "noindex");
}

async fn body(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn page_finishes_while_unread_endpoint_is_blocked() {
    let unread = Arc::new(ControlledUnread::default());
    let (router, _, person, _, _, token, _) =
        setup_conversation_app_with_unread(Some(unread.clone()));
    let (response, ()) = tokio::join!(router.handle(request("/unread-status", &token)), async {
        unread.entered.notified().await;
        let page = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            router.handle(request("/profile/account", &token)),
        )
        .await
        .unwrap();
        assert_eq!(page.status(), http::StatusCode::OK);
        let html = body(page).await;
        assert!(html.contains("/unread-status.js"));
        assert!(html.contains("data-unread-indicator=\"\" hidden=\"hidden\""));
        assert_eq!(*unread.people.lock().unwrap(), vec![person.id]);
        unread.release.notify_one();
    });
    assert_eq!(response.status(), http::StatusCode::OK);
    assert_private(&response);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body(response).await).unwrap(),
        serde_json::json!({"person_id": person.id.to_string(), "has_unread": true}),
    );
}

#[tokio::test]
async fn failed_unread_endpoint_does_not_fail_the_page() {
    let unread = Arc::new(ControlledUnread {
        fail: true,
        ..ControlledUnread::default()
    });
    let (router, _, person, _, offer, token, _) =
        setup_conversation_app_with_unread(Some(unread.clone()));
    let response = router.handle(request("/unread-status", &token)).await;
    assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    assert_private(&response);
    let page = router.handle(request("/profile/account", &token)).await;
    assert_eq!(page.status(), http::StatusCode::OK);
    let detail = router
        .handle(request(&format!("/offers/{}", offer.slug), &token))
        .await;
    assert_eq!(detail.status(), http::StatusCode::OK);
    assert!(body(detail).await.contains("/unread-status.js"));
    assert_eq!(*unread.people.lock().unwrap(), vec![person.id]);
}

#[tokio::test]
async fn unread_endpoint_rejects_guests_and_invalid_sessions_privately() {
    let (router, _, _, _, _, _, _) = setup_conversation_app();
    for token in ["", "invalid-session"] {
        let response = router.handle(request("/unread-status", token)).await;
        assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
        assert_private(&response);
    }
}

#[tokio::test]
async fn unread_status_uses_session_identity_and_never_renders_layout() {
    let (router, registry, maker, responder, _, maker_token, responder_token) =
        setup_conversation_app();
    let conversation = registry
        .conversations
        .get_or_create(ConversationPair::new(maker.id, responder.id).unwrap())
        .await
        .unwrap();
    registry
        .conversations
        .append(
            conversation.id,
            NewMessage {
                sender_id: responder.id,
                body: MessageBody::parse("An incoming message").unwrap(),
                offer_id: None,
            },
        )
        .await
        .unwrap();
    for (token, person_id, expected) in [
        (maker_token, maker.id, true),
        (responder_token, responder.id, false),
    ] {
        let response = router
            .handle(request(
                &format!("/unread-status?user_id={}", responder.id),
                &token,
            ))
            .await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_private(&response);
        assert_eq!(response.headers()["content-type"], "application/json");
        let status: serde_json::Value = serde_json::from_str(&body(response).await).unwrap();
        assert_eq!(
            status,
            serde_json::json!({"person_id": person_id.to_string(), "has_unread": expected})
        );
    }
    let guest_page = router.handle(request("/auth/sign-in", "")).await;
    assert!(!body(guest_page).await.contains("/unread-status.js"));
}
