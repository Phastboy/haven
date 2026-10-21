//! Tests for conversation aggregate and pagination cursors.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use crate::conversation::cursor::{ConversationCursor, MessageCursor};
use crate::conversation::{
    ConversationId, ConversationPair, MessageBody, MessageId, MessageValidationError,
    SelfConversation,
};
use crate::user::UserId;
use chrono::Utc;

#[test]
fn test_conversation_pair_ordering() {
    let u1 = UserId::generate();
    let u2 = UserId::generate();

    let pair1 = ConversationPair::new(u1, u2).unwrap();
    let pair2 = ConversationPair::new(u2, u1).unwrap();

    assert_eq!(pair1, pair2);
    assert_eq!(pair1.other(u1), Some(u2));
    assert_eq!(pair1.other(u2), Some(u1));
    assert!(pair1.contains(u1));
    assert!(pair1.contains(u2));

    let u3 = UserId::generate();
    assert_eq!(pair1.other(u3), None);
    assert!(!pair1.contains(u3));
}

#[test]
fn test_conversation_pair_self_rejected() {
    let u1 = UserId::generate();
    let err = ConversationPair::new(u1, u1).unwrap_err();
    assert_eq!(err, SelfConversation);
}

#[test]
fn test_message_body_validation() {
    assert_eq!(
        MessageBody::parse("   ").unwrap_err(),
        MessageValidationError::BlankBody
    );

    let valid = MessageBody::parse("Hello, is this still available?").unwrap();
    assert_eq!(valid.as_str(), "Hello, is this still available?");

    let long_str = "a".repeat(2001);
    assert_eq!(
        MessageBody::parse(long_str).unwrap_err(),
        MessageValidationError::BodyTooLong
    );

    let max_str = "a".repeat(2000);
    assert!(MessageBody::parse(max_str).is_ok());
}

#[test]
fn test_conversation_cursor_roundtrip() {
    let now = Utc::now();
    let id = ConversationId::generate();
    let cursor = ConversationCursor::new(now, id);
    let encoded = cursor.encode();
    let decoded = ConversationCursor::decode(&encoded).unwrap();

    assert_eq!(cursor.id, decoded.id);
    assert_eq!(
        cursor.last_message_at.timestamp_micros(),
        decoded.last_message_at.timestamp_micros()
    );
}

#[test]
fn test_message_cursor_roundtrip() {
    let now = Utc::now();
    let id = MessageId::generate();
    let cursor = MessageCursor::new(now, id);
    let encoded = cursor.encode();
    let decoded = MessageCursor::decode(&encoded).unwrap();

    assert_eq!(cursor.id, decoded.id);
    assert_eq!(
        cursor.created_at.timestamp_micros(),
        decoded.created_at.timestamp_micros()
    );
}
