use chrono::{DateTime, Utc};
use rand::TryRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::account::AccountId;
use crate::error::SessionError;

/// Newtype for a session's UUID primary key.
/// Cannot be confused with `AccountId` or `OfferId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generates a new random `SessionId`.
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

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A raw session (or magic link) token — the string that goes in the cookie / URL.
/// MUST NOT be stored in the database. Store `HashedToken` instead.
/// The type system prevents storing a `PlaintextToken` where `HashedToken` is expected.
#[derive(Clone)]
pub struct PlaintextToken(String);

impl std::fmt::Debug for PlaintextToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PlaintextToken([REDACTED])")
    }
}

impl PlaintextToken {
    /// Generate a cryptographically random 32-byte token, base64url-encoded (no padding).
    pub fn generate() -> Result<Self, SessionError> {
        let mut bytes = [0u8; 32];
        rand::rng()
            .try_fill_bytes(&mut bytes)
            .map_err(|_| SessionError::TokenGenerationFailed)?;
        Ok(Self(base64url_encode(&bytes)))
    }

    /// Compute SHA-256 of this token and return a `HashedToken`.
    /// The hash is what gets stored in the database.
    pub fn to_hashed(&self) -> HashedToken {
        let mut hasher = Sha256::new();
        hasher.update(self.0.as_bytes());
        let result = hasher.finalize();
        HashedToken(hex::encode(result))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PlaintextToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// A SHA-256 hex-encoded hash of a `PlaintextToken`.
/// This is what is stored in the database.
/// MUST NOT be sent to the user — the type system prevents it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedToken(String);

impl HashedToken {
    /// Construct from a raw hex string (e.g. when reading back from the DB).
    pub fn from_hex(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for HashedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Hash a plaintext token from a query string (used when verifying magic link).
/// This is a convenience so callers don't need to construct a `PlaintextToken` just to hash.
pub fn hash_token(raw: &str) -> HashedToken {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let result = hasher.finalize();
    HashedToken(hex::encode(result))
}

/// An authenticated session record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub account_id: AccountId,
    pub token_hash: HashedToken,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    /// Is this session still valid (not expired) at the given time?
    #[must_use]
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        now < self.expires_at
    }
}

#[allow(
    clippy::indexing_slicing,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "Self-contained base64url encoder"
)]
fn base64url_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    // Simple base64url without padding using the standard alphabet mapping.
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut i = 0;
    while i + 2 < bytes.len() {
        let b0 = bytes[i] as usize;
        let b1 = bytes[i + 1] as usize;
        let b2 = bytes[i + 2] as usize;
        write!(
            out,
            "{}{}{}{}",
            CHARS[b0 >> 2] as char,
            CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char,
            CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char,
            CHARS[b2 & 0x3f] as char,
        )
        .unwrap();
        i += 3;
    }
    if i + 1 == bytes.len() {
        let b0 = bytes[i] as usize;
        write!(
            out,
            "{}{}",
            CHARS[b0 >> 2] as char,
            CHARS[(b0 & 3) << 4] as char,
        )
        .unwrap();
    } else if i + 2 == bytes.len() {
        let b0 = bytes[i] as usize;
        let b1 = bytes[i + 1] as usize;
        write!(
            out,
            "{}{}{}",
            CHARS[b0 >> 2] as char,
            CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char,
            CHARS[(b1 & 0xf) << 2] as char,
        )
        .unwrap();
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn token_generate_and_hash_roundtrip() {
        let pt = PlaintextToken::generate().unwrap();
        let h1 = pt.to_hashed();
        let h2 = pt.to_hashed();
        // Hashing is deterministic
        assert_eq!(h1.as_str(), h2.as_str());
        // Plaintext and hash are different
        assert_ne!(pt.as_str(), h1.as_str());
    }

    #[test]
    fn two_generated_tokens_are_different() {
        let a = PlaintextToken::generate().unwrap();
        let b = PlaintextToken::generate().unwrap();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn hash_token_matches_to_hashed() {
        let pt = PlaintextToken::generate().unwrap();
        let h1 = pt.to_hashed();
        let h2 = hash_token(pt.as_str());
        assert_eq!(h1.as_str(), h2.as_str());
    }

    #[test]
    fn session_is_valid_at_evaluates_against_injected_time() {
        let now = Utc::now();
        let session = Session {
            id: SessionId::generate(),
            account_id: AccountId::generate(),
            token_hash: HashedToken::from_hex("abc".into()),
            expires_at: now + chrono::Duration::minutes(15),
            created_at: now,
            ip_address: None,
            user_agent: None,
        };
        assert!(session.is_valid_at(now));
        assert!(session.is_valid_at(now + chrono::Duration::minutes(10)));
        assert!(!session.is_valid_at(now + chrono::Duration::minutes(16)));
    }
}
