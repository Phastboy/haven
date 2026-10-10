//! In-memory token-bucket rate limiting per instance.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use topcoat::{
    Result as TopcoatResult,
    context::{Cx, app_context},
    router::{error::too_many_requests, request::client_ip},
};

const SWEEP_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct State {
    /// key -> (tokens, last update)
    buckets: HashMap<String, (f64, Instant)>,
    last_sweep: Instant,
}

/// Errors that can occur when configuring a rate limiter.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RateLimiterError {
    #[error("rate limiter capacity must be greater than zero")]
    ZeroCapacity,
    #[error("rate limiter refill rate must be positive and finite")]
    InvalidRefillRate,
}

#[derive(Debug)]
pub struct RateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    state: Mutex<State>,
}

impl RateLimiter {
    /// Attempts to create a new rate limiter, returning an error if parameters are invalid.
    pub fn try_new(capacity: u32, refill_per_sec: f64) -> Result<Self, RateLimiterError> {
        if capacity == 0 {
            return Err(RateLimiterError::ZeroCapacity);
        }
        if !refill_per_sec.is_finite() || refill_per_sec <= 0.0 {
            return Err(RateLimiterError::InvalidRefillRate);
        }
        Ok(Self {
            capacity: f64::from(capacity),
            refill_per_sec,
            state: Mutex::new(State {
                buckets: HashMap::new(),
                last_sweep: Instant::now(),
            }),
        })
    }

    /// Creates a new rate limiter. Clamps non-positive values gracefully without panicking.
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        let valid_capacity = capacity.max(1);
        let valid_refill = if refill_per_sec.is_finite() && refill_per_sec > 0.0 {
            refill_per_sec
        } else {
            1.0
        };
        Self {
            capacity: f64::from(valid_capacity),
            refill_per_sec: valid_refill,
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

impl Default for SignInLimiter {
    fn default() -> Self {
        Self::new()
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

impl Default for CreateOfferLimiter {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_capacity_exhaustion_and_refill() {
        let limiter = RateLimiter::new(2, 10.0);
        assert!(limiter.try_acquire("user-1").is_ok());
        assert!(limiter.try_acquire("user-1").is_ok());
        assert!(limiter.try_acquire("user-1").is_err());
        // Different key is unaffected
        assert!(limiter.try_acquire("user-2").is_ok());
    }

    #[test]
    fn rate_limiter_try_new_validates_inputs() {
        assert!(matches!(
            RateLimiter::try_new(0, 1.0),
            Err(RateLimiterError::ZeroCapacity)
        ));
        assert!(matches!(
            RateLimiter::try_new(5, 0.0),
            Err(RateLimiterError::InvalidRefillRate)
        ));
        assert!(matches!(
            RateLimiter::try_new(5, -1.0),
            Err(RateLimiterError::InvalidRefillRate)
        ));
        assert!(matches!(
            RateLimiter::try_new(5, f64::NAN),
            Err(RateLimiterError::InvalidRefillRate)
        ));
        assert!(RateLimiter::try_new(5, 1.0).is_ok());
    }

    #[test]
    fn rate_limiter_new_does_not_panic_on_invalid_inputs() {
        let limiter = RateLimiter::new(0, -5.0);
        assert!(limiter.try_acquire("key").is_ok());
    }
}
