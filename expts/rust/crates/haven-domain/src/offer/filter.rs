//! Validated public criteria and collection scope; no HTTP or SQL dependencies.

mod price;
mod public;
mod scope;
mod title;

pub use price::PriceFilter;
pub use public::{PublicCriteria, PublicCriterion};
pub use scope::Scope;
pub use title::SearchQuery;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;
