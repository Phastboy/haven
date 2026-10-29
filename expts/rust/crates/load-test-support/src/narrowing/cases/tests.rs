//! Independent fixture-oracle regressions for authenticated and empty histories.
use super::*;
use chrono::{DateTime, Utc};
use haven_domain::offer::PersonalScope;
use haven_domain::user::UserId;
use std::collections::HashMap;
use uuid::Uuid;

fn rows() -> Vec<TruthRow> {
    (1..=25)
        .map(|number| TruthRow {
            id: Uuid::from_u128(number),
            user_id: Uuid::from_u128(100),
            slug: format!("test-offer-{number}"),
            title: "Test Offer".into(),
            category: "food-groceries".into(),
            price: 1002,
            currency: "NGN".into(),
            image_count: 1,
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        })
        .collect()
}

fn actor(token: Option<String>) -> Actor {
    Actor {
        scope: None,
        history: if token.is_some() { "zero" } else { "none" },
        person_id: token.as_ref().map(|_| Uuid::from_u128(200).to_string()),
        token,
        positions: None,
        number: 0,
    }
}

#[test]
fn authenticated_public_preserves_public_truth_and_cursor_semantics() {
    let rows = rows();
    let guest = for_actor(&actor(None), &rows);
    let signed_in = for_actor(&actor(Some("private-token".into())), &rows);
    assert_eq!(guest.len(), signed_in.len());
    for (guest, signed_in) in guest.iter().zip(&signed_in) {
        assert_eq!(guest.expected_slugs, signed_in.expected_slugs);
        assert_eq!(guest.path, signed_in.path);
        assert_eq!(guest.expected_next, signed_in.expected_next);
        assert_eq!(guest.canonical, signed_in.canonical);
        assert_eq!(signed_in.class, "public-authenticated");
        assert_eq!(signed_in.expected_unread, Some(false));
        assert!(signed_in.person_id.is_some());
        assert!(!signed_in.path.contains("scope=public-authenticated"));
        assert!(guest.person_id.is_none());
        assert!(guest.expected_unread.is_none());
    }
    assert!(signed_in.iter().any(|case| case.path.contains("after=")));
}

#[test]
fn zero_history_personal_truth_is_empty_for_every_facet() {
    let mut actor = actor(Some("private-token".into()));
    actor.scope = Some(PersonalScope::Saved(UserId::from_uuid(Uuid::from_u128(
        200,
    ))));
    actor.positions = Some(HashMap::new());
    let cases = for_actor(&actor, &rows());
    assert_eq!(cases.len(), 6);
    for case in cases {
        assert_eq!(case.expected_slugs, Vec::<String>::new());
        assert!(!case.expected_next);
        assert!(!case.canonical);
        assert_eq!(case.history, "zero");
        assert_eq!(case.expected_unread, Some(false));
        assert!(case.path.contains("scope=saved"));
        assert!(!case.path.contains("after="));
    }
}
