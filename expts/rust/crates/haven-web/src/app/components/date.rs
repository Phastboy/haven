//! Shared date labels used in person identity views.

/// Formats a person's creation date for profile and maker attribution.
#[must_use]
pub fn joined_date(created_at: &chrono::DateTime<chrono::Utc>) -> String {
    created_at.format("Joined %B %Y").to_string()
}
