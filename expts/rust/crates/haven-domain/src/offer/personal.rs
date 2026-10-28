//! Authenticated discovery scopes and cursors bound to a person and normalized facets.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{PublicCriteria, Scope, UserId};
use crate::error::CursorError;

/// A personal collection whose actor must come from the authenticated session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalScope {
    /// Available offers saved by this person, ordered by save time.
    Saved(UserId),
    /// Available offers this person messaged about, ordered by latest inquiry.
    Inquired(UserId),
}

impl PersonalScope {
    /// URL representation of this collection, without exposing a person identifier.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Saved(_) => "saved",
            Self::Inquired(_) => "inquired",
        }
    }

    /// Session-resolved owner of the personal collection.
    #[must_use]
    pub const fn person(self) -> UserId {
        match self {
            Self::Saved(person) | Self::Inquired(person) => person,
        }
    }

    /// Shared collection vocabulary, without admitting Public into personal queries.
    #[must_use]
    pub const fn scope(self) -> Scope {
        match self {
            Self::Saved(person) => Scope::Saved(person),
            Self::Inquired(person) => Scope::Inquired(person),
        }
    }
}

/// Stable activity boundary: saved-row id for Saved, offer id for Inquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityPosition {
    /// Save time or latest qualifying inquiry time.
    pub at: DateTime<Utc>,
    /// Scope-specific ordering tiebreaker.
    pub id: Uuid,
}

/// An opaque personal cursor, bound to its actor, scope and normalized facets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalCursor {
    position: ActivityPosition,
    scope: PersonalScope,
    binding: String,
}

impl PersonalCursor {
    /// Creates a boundary for the exact collection and criteria being paged.
    #[must_use]
    pub fn new(
        scope: PersonalScope,
        criteria: &PublicCriteria,
        position: ActivityPosition,
    ) -> Self {
        Self {
            position,
            scope,
            binding: binding(scope, criteria),
        }
    }

    /// Scope-specific timestamp and tiebreaker used by the repository.
    #[must_use]
    pub const fn position(&self) -> ActivityPosition {
        self.position
    }

    /// Encodes a cursor without making its person identifier a query parameter.
    #[must_use]
    pub fn encode(&self) -> String {
        hex::encode(format!(
            "v1~{}~{}~{}~{}",
            self.scope.label(),
            self.binding,
            self.position.at.timestamp_micros(),
            self.position.id
        ))
    }

    /// Decodes only a cursor belonging to the authenticated collection and facets.
    ///
    /// # Errors
    /// Returns [`CursorError::InvalidCursor`] for malformed or differently bound cursors.
    pub fn decode(
        raw: &str,
        scope: PersonalScope,
        criteria: &PublicCriteria,
    ) -> Result<Self, CursorError> {
        if raw.len() > 512 {
            return Err(CursorError::InvalidCursor);
        }
        let bytes = hex::decode(raw).map_err(|_| CursorError::InvalidCursor)?;
        let decoded = String::from_utf8(bytes).map_err(|_| CursorError::InvalidCursor)?;
        let mut parts = decoded.split('~');
        let expected = binding(scope, criteria);
        if parts.next() != Some("v1")
            || parts.next() != Some(scope.label())
            || parts.next() != Some(expected.as_str())
        {
            return Err(CursorError::InvalidCursor);
        }
        let micros = parts
            .next()
            .ok_or(CursorError::InvalidCursor)?
            .parse::<i64>()
            .map_err(|_| CursorError::InvalidCursor)?;
        let at = DateTime::from_timestamp_micros(micros).ok_or(CursorError::InvalidCursor)?;
        let id = parts
            .next()
            .ok_or(CursorError::InvalidCursor)?
            .parse::<Uuid>()
            .map_err(|_| CursorError::InvalidCursor)?;
        if parts.next().is_some() {
            return Err(CursorError::InvalidCursor);
        }
        Ok(Self::new(scope, criteria, ActivityPosition { at, id }))
    }
}

/// Validated personal query, kept separate from the public-only repository API.
#[derive(Debug, Clone)]
pub struct PersonalFeedQuery {
    scope: PersonalScope,
    criteria: PublicCriteria,
    cursor: Option<PersonalCursor>,
}

impl PersonalFeedQuery {
    /// Begins the first page for an authenticated person's collection.
    #[must_use]
    pub const fn new(scope: PersonalScope, criteria: PublicCriteria) -> Self {
        Self {
            scope,
            criteria,
            cursor: None,
        }
    }

    /// Applies a cursor only when its scope, person and normalized facets match.
    ///
    /// # Errors
    /// Returns [`CursorError::InvalidCursor`] rather than reinterpreting another collection's boundary.
    pub fn after(mut self, cursor: PersonalCursor) -> Result<Self, CursorError> {
        if cursor.scope != self.scope || cursor.binding != binding(self.scope, &self.criteria) {
            return Err(CursorError::InvalidCursor);
        }
        self.cursor = Some(cursor);
        Ok(self)
    }

    /// Authenticated collection, excluding Public by construction.
    #[must_use]
    pub const fn scope(&self) -> PersonalScope {
        self.scope
    }
    /// Facets applied before the repository's LIMIT.
    #[must_use]
    pub const fn criteria(&self) -> &PublicCriteria {
        &self.criteria
    }
    /// Validated activity boundary, when paging.
    #[must_use]
    pub fn position(&self) -> Option<ActivityPosition> {
        self.cursor.as_ref().map(PersonalCursor::position)
    }
}

fn binding(scope: PersonalScope, criteria: &PublicCriteria) -> String {
    let category = criteria.category.map_or("", super::Category::as_str);
    let title = criteria
        .title
        .as_ref()
        .map_or_else(String::new, |title| title.as_str().to_lowercase());
    let price = criteria.price.map_or_else(String::new, |price| {
        format!(
            "{}:{}:{}",
            price.currency.as_str(),
            price.min.map_or(0, |value| value.as_i32()),
            price.max.map_or(i32::MAX, |value| value.as_i32())
        )
    });
    hex::encode(Sha256::digest(format!(
        "v1|{}|{}|{category}|{}|{price}",
        scope.label(),
        scope.person().as_uuid(),
        hex::encode(title)
    )))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "personal cursor assertions")]
mod tests;
