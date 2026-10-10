use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AccountError;

/// Newtype wrapping a UUID that identifies an Account.
/// Cannot be confused with `OfferId`, `SessionId`, or `UserId` at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(Uuid);

impl AccountId {
    /// Generates a new random `AccountId`.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for AccountId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A validated email address.
/// Constructed via `Email::parse()` — after construction, guaranteed non-empty and contains '@'.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = AccountError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> Self {
        email.0
    }
}

impl Email {
    /// Parse and validate a raw email string.
    /// Normalises to lowercase + trims whitespace.
    pub fn parse(raw: &str) -> Result<Self, AccountError> {
        let normalised = raw.trim().to_lowercase();
        if normalised.is_empty() {
            return Err(AccountError::InvalidEmail("email must not be empty".into()));
        }
        if !normalised.contains('@') {
            return Err(AccountError::InvalidEmail("email must contain '@'".into()));
        }
        // Split on '@'; local part and domain must both be non-empty.
        let mut parts = normalised.splitn(2, '@');
        let local = parts.next().unwrap_or("");
        let domain = parts.next().unwrap_or("");
        if local.is_empty() || domain.is_empty() {
            return Err(AccountError::InvalidEmail(
                "email local-part and domain must not be empty".into(),
            ));
        }
        Ok(Self(normalised))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The authentication identity record.
/// An Account represents "someone controls this email address."
/// It is NOT a platform participant — that is `User`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub email: Email,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn email_normalises_whitespace_and_case() {
        let e = Email::parse("  Hello@Example.COM  ").unwrap();
        assert_eq!(e.as_str(), "hello@example.com");
    }

    #[test]
    fn email_rejects_no_at_sign() {
        assert!(Email::parse("notanemail").is_err());
    }

    #[test]
    fn email_rejects_empty() {
        assert!(Email::parse("").is_err());
    }

    #[test]
    fn email_rejects_missing_local_part() {
        assert!(Email::parse("@example.com").is_err());
    }

    #[test]
    fn email_rejects_missing_domain() {
        assert!(Email::parse("user@").is_err());
    }

    #[test]
    fn account_id_is_unique() {
        let a = AccountId::generate();
        let b = AccountId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn email_serde_validates_on_deserialization() {
        assert!(serde_json::from_str::<Email>("\"notanemail\"").is_err());
        assert!(serde_json::from_str::<Email>("\"\"").is_err());
        let valid: Email = serde_json::from_str("\"user@example.com\"").unwrap();
        assert_eq!(valid.as_str(), "user@example.com");
    }
}
