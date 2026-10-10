//! URL-safe unique offer slug value object and derivation.

use serde::{Deserialize, Serialize};

use crate::error::OfferValidationError;

/// Minimum character count for an offer slug.
pub const MIN_SLUG_CHARS: usize = 3;

/// Maximum character count for an offer slug.
pub const MAX_SLUG_CHARS: usize = 120;

/// Slugs reserved by the routing structure that cannot identify an offer.
pub const RESERVED_SLUGS: &[&str] = &["manage", "new"];

/// A URL-safe unique slug for public offer lookup.
/// Conforms to lowercase alphanumeric segments separated by hyphens (e.g. `vintage-chair-a1b2c3d4`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OfferSlug(String);

impl TryFrom<String> for OfferSlug {
    type Error = OfferValidationError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<OfferSlug> for String {
    fn from(s: OfferSlug) -> Self {
        s.0
    }
}

impl OfferSlug {
    /// Validates and parses a raw slug string.
    ///
    /// # Errors
    ///
    /// Returns [`OfferValidationError::InvalidSlug`] if the string violates length bounds,
    /// format constraints, or matches a reserved route name.
    pub fn parse(s: &str) -> Result<Self, OfferValidationError> {
        let trimmed = s.trim();
        let len = trimmed.len();
        if !(MIN_SLUG_CHARS..=MAX_SLUG_CHARS).contains(&len) {
            return Err(OfferValidationError::InvalidSlug(s.to_string()));
        }

        if RESERVED_SLUGS.contains(&trimmed) {
            return Err(OfferValidationError::InvalidSlug(s.to_string()));
        }

        let mut prev_hyphen = false;
        for (i, c) in trimmed.chars().enumerate() {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                prev_hyphen = false;
            } else if c == '-' {
                if i == 0 || prev_hyphen {
                    return Err(OfferValidationError::InvalidSlug(s.to_string()));
                }
                prev_hyphen = true;
            } else {
                return Err(OfferValidationError::InvalidSlug(s.to_string()));
            }
        }

        if prev_hyphen {
            return Err(OfferValidationError::InvalidSlug(s.to_string()));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Derives a clean URL-safe slug from a title and a unique suffix (e.g. offer id prefix).
    ///
    /// # Errors
    ///
    /// Returns [`OfferValidationError::InvalidSlug`] if the derived slug violates slug format.
    pub fn from_title_and_suffix(title: &str, suffix: &str) -> Result<Self, OfferValidationError> {
        let mut base = String::new();
        let mut last_was_hyphen = false;
        for c in title.trim().chars() {
            if c.is_ascii_alphanumeric() {
                base.push(c.to_ascii_lowercase());
                last_was_hyphen = false;
            } else if !last_was_hyphen && !base.is_empty() {
                base.push('-');
                last_was_hyphen = true;
            }
        }
        let mut base_clean = base.trim_end_matches('-').to_string();
        if base_clean.is_empty() {
            base_clean = "offer".to_string();
        }
        if base_clean.len() > 80 {
            let mut truncated = String::new();
            for c in base_clean.chars().take(80) {
                truncated.push(c);
            }
            base_clean = truncated.trim_end_matches('-').to_string();
            if base_clean.is_empty() {
                base_clean = "offer".to_string();
            }
        }

        let mut suffix_clean = String::new();
        for c in suffix
            .to_ascii_lowercase()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .take(8)
        {
            suffix_clean.push(c);
        }
        let suffix_val = if suffix_clean.is_empty() {
            "00000000"
        } else {
            &suffix_clean
        };

        let combined = format!("{base_clean}-{suffix_val}");
        Self::parse(&combined)
    }

    /// Returns the slug as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OfferSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
