//! Row mapping from database records to domain offer models.

use haven_domain::offer::{CurrencyCode, Offer, OfferId, OfferSlug, Price, UserId};
use haven_domain::ports::RepoError;
use uuid::Uuid;

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    reason = "Internal DB mapping function"
)]
pub(super) fn map_row(
    id: Uuid,
    user_id: Uuid,
    slug: String,
    title: String,
    description: String,
    price: i32,
    currency: &str,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Result<Offer, RepoError> {
    let p = Price::new(price).map_err(|_| RepoError::Corrupt("Invalid price".into()))?;
    let c =
        CurrencyCode::parse(currency).map_err(|_| RepoError::Corrupt("Invalid currency".into()))?;
    let s = OfferSlug::parse(&slug).map_err(|_| RepoError::Corrupt("Invalid slug".into()))?;

    Ok(Offer {
        id: OfferId::from_uuid(id),
        user_id: UserId::from_uuid(user_id),
        slug: s,
        title,
        description,
        price: p,
        currency: c,
        created_at,
        updated_at,
    })
}
