use super::*;
use crate::offer::{Category, CurrencyCode, PriceFilter};

#[test]
fn personal_cursor_round_trips_and_rejects_other_people_scopes_and_facets() {
    let person = UserId::generate();
    let scope = PersonalScope::Saved(person);
    let criteria = PublicCriteria {
        category: Some(Category::FoodGroceries),
        ..PublicCriteria::default()
    };
    let position = ActivityPosition {
        at: Utc::now(),
        id: Uuid::new_v4(),
    };
    let cursor = PersonalCursor::new(scope, &criteria, position);
    let encoded = cursor.encode();
    let decoded = PersonalCursor::decode(&encoded, scope, &criteria).unwrap();
    assert_eq!(decoded.position().id, position.id);
    assert_eq!(
        decoded.position().at.timestamp_micros(),
        position.at.timestamp_micros()
    );
    assert!(
        PersonalCursor::decode(
            &encoded,
            PersonalScope::Saved(UserId::generate()),
            &criteria
        )
        .is_err()
    );
    assert!(PersonalCursor::decode(&encoded, PersonalScope::Inquired(person), &criteria).is_err());
    assert!(PersonalCursor::decode(&encoded, scope, &PublicCriteria::default()).is_err());
    assert!(PersonalCursor::decode("broken", scope, &criteria).is_err());
    assert!(
        PersonalFeedQuery::new(PersonalScope::Inquired(person), criteria.clone())
            .after(cursor)
            .is_err()
    );
}

#[test]
fn normalized_price_bounds_bind_to_the_same_collection() {
    let scope = PersonalScope::Inquired(UserId::generate());
    let criteria = PublicCriteria {
        price: Some(PriceFilter::free(CurrencyCode::Ngn)),
        ..PublicCriteria::default()
    };
    let cursor = PersonalCursor::new(
        scope,
        &criteria,
        ActivityPosition {
            at: Utc::now(),
            id: Uuid::new_v4(),
        },
    );
    let equivalent = PublicCriteria {
        price: Some(
            PriceFilter::new(CurrencyCode::Ngn, None, Some(crate::offer::Price::ZERO)).unwrap(),
        ),
        ..PublicCriteria::default()
    };
    assert!(PersonalCursor::decode(&cursor.encode(), scope, &equivalent).is_ok());
}
