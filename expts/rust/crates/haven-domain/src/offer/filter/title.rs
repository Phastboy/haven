//! Validated title search criterion.

use crate::error::FilterValidationError;
use serde::{Deserialize, Serialize};

/// Validated search query string for matching offer titles.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// Maximum character length for a title search query.
    pub const MAX_CHARS: usize = 100;

    /// Validates and constructs a search query.
    ///
    /// # Errors
    ///
    /// Returns [`FilterValidationError::BlankSearchQuery`] if `raw` is empty after trimming,
    /// or [`FilterValidationError::SearchQueryTooLong`] if it exceeds [`Self::MAX_CHARS`].
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, FilterValidationError> {
        let trimmed = raw.as_ref().trim();
        if trimmed.is_empty() {
            return Err(FilterValidationError::BlankSearchQuery);
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(FilterValidationError::SearchQueryTooLong);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the underlying raw query string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
