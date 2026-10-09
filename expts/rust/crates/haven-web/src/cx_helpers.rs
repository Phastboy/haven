//! Composable request helpers (ENGINEERING.md: functions, not middlewares).
//!
//! Depends on two `haven-db` functions that must be added:
//!   - `haven_db::users::find_by_session(pool, token_hash: &str) -> sqlx::Result<Option<User>>`
//!     joins session -> user via `account_id`, and checks `expires_at > now()` in SQL
//!   - `haven_db::offers::find_owned(pool, OfferId, UserId) -> sqlx::Result<Option<Offer>>`
//!     `WHERE id = $1 AND user_id = $2`

use std::{
    collections::HashMap,
    fmt,
    sync::Mutex,
    time::{Duration, Instant},
};

use haven_domain::{
    offer::{Offer, OfferId},
    user::User,
};
use topcoat::{
    Error as RouterError, Result as TopcoatResult,
    context::{Cx, app_context, memoize},
    router::{
        error::{RouterErrorExt, too_many_requests},
        request::client_ip,
    },
    session,
};

// ---------------------------------------------------------------------------
// App context accessors
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub registry: std::sync::Arc<dyn haven_domain::ports::Registry>,
    pub google_oauth: Option<crate::app::auth::google::GoogleOAuthConfig>,
    pub http_client: reqwest::Client,
}

pub fn registry(cx: &Cx) -> &dyn haven_domain::ports::Registry {
    &*app_context::<AppState>(cx).registry
}

pub fn google_oauth(cx: &Cx) -> Option<&crate::app::auth::google::GoogleOAuthConfig> {
    app_context::<AppState>(cx).google_oauth.as_ref()
}

pub fn http_client(cx: &Cx) -> &reqwest::Client {
    &app_context::<AppState>(cx).http_client
}

pub fn map_repo_err(e: haven_domain::ports::RepoError) -> RouterError {
    use haven_domain::ports::RepoError;
    match e {
        RepoError::NotFound => topcoat::router::error::not_found().into(),
        RepoError::Conflict => topcoat::router::error::bad_request("Conflict").into(),
        RepoError::Corrupt(msg) | RepoError::Unavailable(msg) => topcoat::Error::msg(msg),
    }
}

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Ownership
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Rate limiting (token bucket, in-memory, per instance)
// ---------------------------------------------------------------------------

const SWEEP_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct State {
    /// key -> (tokens, last update)
    buckets: HashMap<String, (f64, Instant)>,
    last_sweep: Instant,
}

#[derive(Debug)]
pub struct RateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    state: Mutex<State>,
}

impl RateLimiter {
    /// `capacity` is the burst size; `refill_per_sec` is the sustained rate.
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        assert!(
            capacity > 0 && refill_per_sec > 0.0,
            "limiter needs a positive burst and refill rate"
        );
        Self {
            capacity: f64::from(capacity),
            refill_per_sec,
            state: Mutex::new(State {
                buckets: HashMap::new(),
                last_sweep: Instant::now(),
            }),
        }
    }

    /// Takes one token for `key`. `Err(retry_after)` when the bucket is empty.
    ///
    /// Buckets that have been idle long enough to be full again carry no
    /// information, so a periodic sweep drops them and memory stays bounded
    /// by the number of recently active keys.
    pub fn try_acquire(&self, key: &str) -> Result<(), Duration> {
        let now = Instant::now();
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let state = &mut *guard;

        if now.duration_since(state.last_sweep) >= SWEEP_INTERVAL {
            let full_after = Duration::from_secs_f64(self.capacity / self.refill_per_sec);
            state
                .buckets
                .retain(|_, v| now.duration_since(v.1) < full_after);
            state.last_sweep = now;
        }

        let (tokens, last) = state
            .buckets
            .entry(key.to_owned())
            .or_insert((self.capacity, now));

        let elapsed = now.duration_since(*last).as_secs_f64();
        *tokens = (*tokens + elapsed * self.refill_per_sec).min(self.capacity);
        *last = now;

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            Err(Duration::from_secs_f64(
                (1.0 - *tokens) / self.refill_per_sec,
            ))
        }
    }
}

// App context is keyed by type, so each limiter gets its own wrapper type.

/// Sign-in: keyed by IP and by email. Burst 5, then one per minute.
#[derive(Debug)]
pub struct SignInLimiter(pub RateLimiter);

impl SignInLimiter {
    pub fn new() -> Self {
        Self(RateLimiter::new(5, 1.0 / 60.0))
    }
}

/// Offer creation: keyed by user id. Burst 5, then one every 4 seconds.
#[derive(Debug)]
pub struct CreateOfferLimiter(pub RateLimiter);

impl CreateOfferLimiter {
    pub fn new() -> Self {
        Self(RateLimiter::new(5, 0.25))
    }
}

pub fn sign_in_limiter(cx: &Cx) -> &RateLimiter {
    &app_context::<SignInLimiter>(cx).0
}

pub fn create_offer_limiter(cx: &Cx) -> &RateLimiter {
    &app_context::<CreateOfferLimiter>(cx).0
}

/// Rate-limit key for the caller's IP. `client_ip` is Topcoat's own resolver:
/// the peer address by default, and the forwarded-header address only when
/// the router is configured with `TrustedProxies`, so the header cannot be
/// spoofed by arbitrary clients. `None` only happens with no peer address
/// (tests); those requests share one conservative bucket.
pub fn client_ip_key(cx: &Cx) -> String {
    client_ip(cx).map_or_else(|| "no-ip".to_owned(), |ip| ip.to_string())
}

/// Takes a token or answers 429 with `Retry-After`. For routes that must
/// refuse (offer creation). Sign-in does NOT use this: it calls
/// `try_acquire` directly and redirects to `/auth/sent` either way, so the
/// limiter cannot reveal whether an email exists.
pub fn enforce(limiter: &RateLimiter, key: &str) -> TopcoatResult<()> {
    limiter
        .try_acquire(key)
        .map_err(|wait| too_many_requests(wait.as_secs().max(1)).into())
}
