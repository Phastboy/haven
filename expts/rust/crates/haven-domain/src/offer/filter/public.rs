//! Composition of the public facets shared by all discovery scopes.

use super::{PriceFilter, SearchQuery};
use serde::{Deserialize, Serialize};

/// Public facets intersected to narrow an offer collection.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicCriteria {
    /// Optional title search query.
    pub title: Option<SearchQuery>,
    /// Optional price and currency filter.
    pub price: Option<PriceFilter>,
}

/// One validated public facet, borrowed for layer-specific composition.
#[derive(Debug, Clone, Copy)]
pub enum PublicCriterion<'a> {
    /// Match an offer title.
    Title(&'a SearchQuery),
    /// Match an offer's currency and inclusive price range.
    Price(&'a PriceFilter),
}

impl PublicCriteria {
    /// Visits active facets in stable title/price order.
    pub fn iter(&self) -> impl Iterator<Item = PublicCriterion<'_>> {
        [
            self.title.as_ref().map(PublicCriterion::Title),
            self.price.as_ref().map(PublicCriterion::Price),
        ]
        .into_iter()
        .flatten()
    }

    /// Returns true when the collection is unconstrained by public facets.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
}
