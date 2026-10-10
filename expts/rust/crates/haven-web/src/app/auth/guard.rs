//! Authentication and authorization guards.

use std::fmt;

use haven_domain::{
    offer::{Offer, OfferId},
    user::User,
};
use topcoat::{
    Error as RouterError, Result as TopcoatResult,
    context::{Cx, memoize},
    router::error::RouterErrorExt,
    session,
};

use crate::app::state::{map_repo_err, registry};

/// A failed auth lookup (database or session-cookie error). Kept as our own
/// `Clone` type because `#[memoize]` hands back a reference to the cached
/// result, and we need to return an owned value from `current_user`.
#[derive(Debug, Clone)]
pub struct AuthLookupFailed;

impl fmt::Display for AuthLookupFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("authentication lookup failed")
    }
}

impl std::error::Error for AuthLookupFailed {}

/// The `session.token_hash` column stores hex text. Topcoat's `TokenHash` is
/// 32 raw bytes (it derefs to `[u8; 32]`). Every read AND write of that column
/// must go through this one encoder (lowercase hex), or lookups will miss.
pub fn token_hash_hex(hash: &session::TokenHash) -> String {
    use std::fmt::Write;
    hash.iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            #[allow(clippy::let_underscore_must_use, reason = "infallible string write")]
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// One session lookup per request, shared by every caller (layout + page).
/// Expiry is enforced inside the SQL, not by the purge job.
#[memoize]
async fn session_user(cx: &Cx) -> Result<Option<User>, AuthLookupFailed> {
    let hash = session::token_hash(cx).await.map_err(|_| {
        // Intentionally swallow error if token hash fails
        AuthLookupFailed
    })?;

    let Some(hash) = hash else {
        return Ok(None);
    };

    let domain_hash = haven_domain::session::HashedToken::from_hex(token_hash_hex(&hash));
    registry(cx)
        .users()
        .find_by_session(&domain_hash)
        .await
        .map_err(|_e| {
            // Intentionally swallow error if user lookup fails
            AuthLookupFailed
        })
}

/// The signed-in user, if any. A DB failure is an error (500), not `None`,
/// so an outage never looks like "everyone got logged out".
pub async fn current_user(cx: &Cx) -> TopcoatResult<Option<User>> {
    match session_user(cx).await {
        Ok(user) => Ok(user.clone()),
        Err(e) => Err(RouterError::from(e.clone())),
    }
}

/// The signed-in user, or a redirect to the sign-in page.
pub async fn require_auth(cx: &Cx) -> TopcoatResult<User> {
    Ok(current_user(cx).await?.ok_or_redirect("/auth/sign-in")?)
}

/// Authenticates the user for owner-only management routes and applies private headers:
/// `Cache-Control: private, no-store` and `X-Robots-Tag: noindex`.
/// If unauthenticated, redirects to `/auth/sign-in`.
pub async fn require_owner_auth(cx: &Cx) -> TopcoatResult<User> {
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        topcoat::router::header::HeaderName::from_static("cache-control"),
        topcoat::router::header::HeaderValue::from_static("private, no-store"),
    );
    headers.append(
        topcoat::router::header::HeaderName::from_static("x-robots-tag"),
        topcoat::router::header::HeaderValue::from_static("noindex"),
    );
    require_auth(cx).await
}

/// The offer, only if it exists AND belongs to the current user. Missing and
/// not-yours are deliberately indistinguishable (404), so IDs cannot be
/// probed. The rule lives in the query, so there is nothing to forget.
pub async fn owned_offer(cx: &Cx, id: OfferId) -> TopcoatResult<Offer> {
    let user = require_owner_auth(cx).await?;

    let offer = registry(cx)
        .offers()
        .find_owned(id, user.id)
        .await
        .map_err(map_repo_err)?;

    Ok(offer.ok_or_else(topcoat::router::error::not_found)?)
}
