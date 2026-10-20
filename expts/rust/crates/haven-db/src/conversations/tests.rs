use super::*;
use haven_domain::conversation::{ConversationPair, MessageBody, NewMessage};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn test_conversation_get_or_create_and_messages(pool: PgPool) {
    let repo = PostgresConversationRepository::new(pool.clone());

    // 1. Create two users
    let a1 = Uuid::new_v4();
    let a2 = Uuid::new_v4();
    sqlx::query("INSERT INTO account (id, email) VALUES ($1, $2), ($3, $4)")
        .bind(a1)
        .bind(format!("{a1}@example.com"))
        .bind(a2)
        .bind(format!("{a2}@example.com"))
        .execute(&pool)
        .await
        .unwrap();

    let u1 = UserId::from_uuid(Uuid::new_v4());
    let u2 = UserId::from_uuid(Uuid::new_v4());
    sqlx::query("INSERT INTO \"user\" (id, account_id, display_name, handle) VALUES ($1, $2, 'User One', $3), ($4, $5, 'User Two', $6)")
        .bind(u1.as_uuid())
        .bind(a1)
        .bind(format!("u1-{}", &u1.as_uuid().to_string()[..6]))
        .bind(u2.as_uuid())
        .bind(a2)
        .bind(format!("u2-{}", &u2.as_uuid().to_string()[..6]))
        .execute(&pool)
        .await
        .unwrap();

    // 2. Get or create conversation from both directions
    let pair1 = ConversationPair::new(u1, u2).unwrap();
    let pair2 = ConversationPair::new(u2, u1).unwrap();

    let conv1 = repo.get_or_create(pair1).await.unwrap();
    let conv2 = repo.get_or_create(pair2).await.unwrap();

    assert_eq!(conv1.id, conv2.id);

    // 3. No unread initially
    assert!(!repo.unread_exists(u1).await.unwrap());
    assert!(!repo.unread_exists(u2).await.unwrap());

    // 4. User 1 appends message
    let body = MessageBody::parse("Hi there!").unwrap();
    let msg = repo
        .append(
            conv1.id,
            NewMessage {
                sender_id: u1,
                body: body.clone(),
                offer_id: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(msg.sender_id, u1);
    assert_eq!(msg.body, body);

    // Sender shouldn't see unread, recipient should!
    assert!(!repo.unread_exists(u1).await.unwrap());
    assert!(repo.unread_exists(u2).await.unwrap());

    // 5. Check inbox for u2
    let inbox = repo.list_for_user(u2, None, 10).await.unwrap();
    assert_eq!(inbox.items.len(), 1);
    assert!(inbox.items[0].is_unread);
    assert_eq!(inbox.items[0].last_message_body, "Hi there!");

    // 6. Mark read for u2
    repo.mark_read(conv1.id, u2).await.unwrap();
    assert!(!repo.unread_exists(u2).await.unwrap());

    let inbox_after = repo.list_for_user(u2, None, 10).await.unwrap();
    assert!(!inbox_after.items[0].is_unread);

    // 7. Check thread messages
    let msgs = repo.messages(conv1.id, None, 10).await.unwrap();
    assert_eq!(msgs.items.len(), 1);
    assert_eq!(msgs.items[0].id, msg.id);
}
