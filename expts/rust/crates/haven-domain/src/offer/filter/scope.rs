//! Membership of an offer collection, independent of public facets.

use crate::user::UserId;

/// The collection to which public criteria apply.
///
/// Only [`Scope::Public`] is currently accepted by the public feed API.
/// Personal retrieval and its ordering/cursors are implemented in later steps.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    /// All publicly visible offers.
    #[default]
    Public,
    /// Offers saved by an authenticated person.
    Saved(UserId),
    /// Offers an authenticated person has inquired about.
    Inquired(UserId),
}
