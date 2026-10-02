use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DomainError;

/// Newtype wrapping a UUID that identifies an Account.
/// Cannot be confused with OfferId, SessionId, or UserId at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub Uuid);

impl AccountId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for AccountId {
    fn default() -> Self {
        Self::new()
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
pub struct Email(String);

impl Email {
    /// Parse and validate a raw email string.
    /// Normalises to lowercase + trims whitespace.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let normalised = raw.trim().to_lowercase();
        if normalised.is_empty() {
            return Err(DomainError::InvalidEmail("email must not be empty".into()));
        }
        if !normalised.contains('@') {
            return Err(DomainError::InvalidEmail(
                "email must contain '@'".into(),
            ));
        }
        // Split on '@'; local part and domain must both be non-empty.
        let mut parts = normalised.splitn(2, '@');
        let local = parts.next().unwrap_or("");
        let domain = parts.next().unwrap_or("");
        if local.is_empty() || domain.is_empty() {
            return Err(DomainError::InvalidEmail(
                "email local-part and domain must not be empty".into(),
            ));
        }
        Ok(Self(normalised))
    }

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
        let a = AccountId::new();
        let b = AccountId::new();
        assert_ne!(a, b);
    }
}
