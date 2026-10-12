//! User entity and participant identifier.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::account::AccountId;

/// Newtype wrapping a UUID that identifies a User (platform participant).
/// Cannot be confused with `AccountId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Generates a new random `UserId`.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs a `UserId` from an existing UUID.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying raw UUID value.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Minimum length for a display name.
pub const MIN_DISPLAY_NAME_CHARS: usize = 2;
/// Maximum length for a display name.
pub const MAX_DISPLAY_NAME_CHARS: usize = 50;

/// Minimum length for a profile handle.
pub const MIN_HANDLE_CHARS: usize = 3;
/// Maximum length for a profile handle.
pub const MAX_HANDLE_CHARS: usize = 30;

/// Maximum length for a bio.
pub const MAX_BIO_CHARS: usize = 500;

/// Reserved handle keywords disallowed for public user profiles.
pub const RESERVED_HANDLES: &[&str] = &[
    "admin",
    "api",
    "app",
    "auth",
    "dashboard",
    "edit",
    "help",
    "login",
    "logout",
    "manage",
    "new",
    "offers",
    "profile",
    "profiles",
    "root",
    "settings",
    "signin",
    "signout",
    "signup",
    "static",
    "style",
    "verify",
];

pub use crate::error::ProfileError;

/// Validated unique URL-safe handle (username) identifying a person.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProfileHandle(String);

impl ProfileHandle {
    /// Parses and validates a profile handle.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] if too short, too long, contains invalid characters,
    /// has consecutive/leading/trailing hyphens, or matches a reserved keyword.
    pub fn parse(raw: &str) -> Result<Self, ProfileError> {
        let trimmed = raw.trim();
        let lower = trimmed.to_ascii_lowercase();
        let char_count = lower.chars().count();
        if char_count < MIN_HANDLE_CHARS {
            return Err(ProfileError::HandleTooShort);
        }
        if char_count > MAX_HANDLE_CHARS {
            return Err(ProfileError::HandleTooLong);
        }

        let mut prev_hyphen = false;
        for (i, ch) in lower.chars().enumerate() {
            if ch.is_ascii_alphanumeric() {
                prev_hyphen = false;
            } else if ch == '-' {
                if i == 0 || i == char_count.saturating_sub(1) || prev_hyphen {
                    return Err(ProfileError::InvalidHandleFormat);
                }
                prev_hyphen = true;
            } else {
                return Err(ProfileError::InvalidHandleFormat);
            }
        }

        if RESERVED_HANDLES.contains(&lower.as_str()) {
            return Err(ProfileError::ReservedHandle(lower));
        }

        Ok(Self(lower))
    }

    /// Returns the handle as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generates an initial handle derived from an email address and a short random suffix.
    #[must_use]
    pub fn generate_initial(raw_email: &str, suffix: &str) -> Self {
        let local_part = raw_email.split('@').next().unwrap_or("user");
        let mut sanitized = String::with_capacity(30);
        let mut prev_hyphen = false;
        for ch in local_part.chars() {
            if ch.is_ascii_alphanumeric() {
                sanitized.push(ch.to_ascii_lowercase());
                prev_hyphen = false;
            } else if !prev_hyphen && !sanitized.is_empty() {
                sanitized.push('-');
                prev_hyphen = true;
            }
        }
        let trimmed_base = sanitized.trim_matches('-');
        let base = if trimmed_base.is_empty() {
            "user"
        } else {
            trimmed_base
        };
        let max_base_len = MAX_HANDLE_CHARS.saturating_sub(suffix.len().saturating_add(1));
        let truncated: String = base.chars().take(max_base_len).collect();
        let candidate = format!("{}-{suffix}", truncated.trim_matches('-'));
        Self::parse(&candidate).unwrap_or_else(|_| {
            let fallback = format!("user-{suffix}");
            Self(fallback)
        })
    }
}

impl TryFrom<String> for ProfileHandle {
    type Error = ProfileError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<ProfileHandle> for String {
    fn from(handle: ProfileHandle) -> Self {
        handle.0
    }
}

impl std::fmt::Display for ProfileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Validated display name representing a person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DisplayName(String);

impl DisplayName {
    /// Parses and validates a display name.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] if blank, shorter than 2 characters, or longer than 50 characters.
    pub fn parse(raw: &str) -> Result<Self, ProfileError> {
        let trimmed = raw.trim();
        let char_count = trimmed.chars().count();
        if char_count == 0 {
            return Err(ProfileError::BlankDisplayName);
        }
        if char_count < MIN_DISPLAY_NAME_CHARS {
            return Err(ProfileError::DisplayNameTooShort);
        }
        if char_count > MAX_DISPLAY_NAME_CHARS {
            return Err(ProfileError::DisplayNameTooLong);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the display name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DisplayName {
    type Error = ProfileError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<DisplayName> for String {
    fn from(name: DisplayName) -> Self {
        name.0
    }
}

impl std::fmt::Display for DisplayName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Validated optional bio (about section) for a person's profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Bio(String);

impl Bio {
    /// Parses and validates a bio.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] if longer than 500 characters.
    pub fn parse(raw: &str) -> Result<Self, ProfileError> {
        let trimmed = raw.trim();
        let char_count = trimmed.chars().count();
        if char_count > MAX_BIO_CHARS {
            return Err(ProfileError::BioTooLong);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the bio as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Bio {
    type Error = ProfileError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Bio> for String {
    fn from(bio: Bio) -> Self {
        bio.0
    }
}

impl std::fmt::Display for Bio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Input data for updating a person's profile.
#[derive(Debug, Clone)]
pub struct UserProfileUpdate {
    /// Proposed display name.
    pub display_name: DisplayName,
    /// Proposed unique handle.
    pub handle: ProfileHandle,
    /// Proposed optional bio.
    pub bio: Option<Bio>,
}

impl UserProfileUpdate {
    /// Constructs initial profile data derived from an email address and a random suffix.
    #[must_use]
    pub fn from_email_initial(raw_email: &str, suffix: &str) -> Self {
        let handle = ProfileHandle::generate_initial(raw_email, suffix);
        let local_part = raw_email.split('@').next().unwrap_or("User");
        let display_name = DisplayName::parse(local_part)
            .or_else(|_| DisplayName::parse(handle.as_str()))
            .unwrap_or_else(|_| DisplayName("User".to_string()));
        Self {
            display_name,
            handle,
            bio: None,
        }
    }
}

/// Platform participant (authz + platform context + public identity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier of the participant.
    pub id: UserId,
    /// Associated account identifier.
    pub account_id: AccountId,
    /// Unique public handle used in profile URLs.
    pub handle: ProfileHandle,
    /// Display name shown on offers and profiles.
    pub display_name: DisplayName,
    /// Optional bio or about section.
    pub bio: Option<Bio>,
    /// Timestamp when this participant was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this participant was last updated.
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_handle_validation() {
        assert!(ProfileHandle::parse("alex").is_ok());
        assert!(ProfileHandle::parse("jane-doe").is_ok());
        assert!(ProfileHandle::parse("user-1234").is_ok());

        // Too short
        assert_eq!(
            ProfileHandle::parse("al"),
            Err(ProfileError::HandleTooShort)
        );
        // Too long
        assert_eq!(
            ProfileHandle::parse("a".repeat(31).as_str()),
            Err(ProfileError::HandleTooLong)
        );
        // Reserved
        assert_eq!(
            ProfileHandle::parse("admin"),
            Err(ProfileError::ReservedHandle("admin".into()))
        );
        assert_eq!(
            ProfileHandle::parse("offers"),
            Err(ProfileError::ReservedHandle("offers".into()))
        );
        // Invalid characters / hyphens
        assert_eq!(
            ProfileHandle::parse("-alex"),
            Err(ProfileError::InvalidHandleFormat)
        );
        assert_eq!(
            ProfileHandle::parse("alex-"),
            Err(ProfileError::InvalidHandleFormat)
        );
        assert_eq!(
            ProfileHandle::parse("alex--doe"),
            Err(ProfileError::InvalidHandleFormat)
        );
        assert_eq!(
            ProfileHandle::parse("alex doe"),
            Err(ProfileError::InvalidHandleFormat)
        );
    }

    #[test]
    fn test_display_name_validation() {
        assert!(DisplayName::parse("Alex Smith").is_ok());
        assert!(DisplayName::parse("Bo").is_ok());

        assert_eq!(DisplayName::parse(""), Err(ProfileError::BlankDisplayName));
        assert_eq!(
            DisplayName::parse("   "),
            Err(ProfileError::BlankDisplayName)
        );
        assert_eq!(
            DisplayName::parse("A"),
            Err(ProfileError::DisplayNameTooShort)
        );
        assert_eq!(
            DisplayName::parse("A".repeat(51).as_str()),
            Err(ProfileError::DisplayNameTooLong)
        );
    }

    #[test]
    fn test_bio_validation() {
        assert!(Bio::parse("").is_ok());
        assert!(Bio::parse("Hello, I craft woodwork.").is_ok());
        assert_eq!(
            Bio::parse("x".repeat(501).as_str()),
            Err(ProfileError::BioTooLong)
        );
    }

    #[test]
    fn test_generate_initial_handle() {
        let handle = ProfileHandle::generate_initial("alex.smith@example.com", "a1b2");
        assert_eq!(handle.as_str(), "alex-smith-a1b2");

        let handle2 = ProfileHandle::generate_initial("a@example.com", "9999");
        assert_eq!(handle2.as_str(), "a-9999");

        // High entropy 8-character hex suffix
        let handle3 = ProfileHandle::generate_initial("alex.doe@example.com", "7a1b3c9d");
        assert_eq!(handle3.as_str(), "alex-doe-7a1b3c9d");
    }

    #[test]
    fn test_user_profile_update_from_email_initial() {
        let update = UserProfileUpdate::from_email_initial("alex.doe@example.com", "7a1b3c9d");
        assert_eq!(update.display_name.as_str(), "alex.doe");
        assert_eq!(update.handle.as_str(), "alex-doe-7a1b3c9d");
        assert!(update.bio.is_none());

        let fallback = UserProfileUpdate::from_email_initial("a@example.com", "12345678");
        assert_eq!(fallback.display_name.as_str(), "a-12345678");
        assert_eq!(fallback.handle.as_str(), "a-12345678");
    }
}
