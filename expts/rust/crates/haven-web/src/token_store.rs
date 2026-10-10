//! Adaptive session cookie token store.
//!
//! Provides an RFC 6265bis-hardened session cookie store that adapts dynamically
//! to TLS vs non-TLS environments:
//!
//! - In HTTPS/TLS environments (e.g. production, staging, reverse proxies setting
//!   `X-Forwarded-Proto: https`): cookies use the `__Host-` prefix and the `Secure`
//!   attribute.
//! - In plain HTTP environments (e.g. LAN testing on non-localhost IPs such as
//!   `192.168.0.50.nip.io`): cookies omit the `__Host-` prefix and `Secure` flag so
//!   standard browsers (Firefox, Chrome, Safari) do not drop them per RFC 6265bis.
//! - Reads transparently accept either `__Host-sid` or `sid` (preferring `__Host-sid`).
//! - Deletions clear both forms to prevent lingering sessions.

use std::{borrow::Cow, time::Duration};

use topcoat::{
    context::{Cx, try_request_context},
    cookie::{Cookie, Cookies, SameSite},
    session::{Token, TokenStore, TokenStoreFuture},
};

/// Default base name for the session cookie (without prefix).
pub const SESSION_COOKIE_NAME: &str = "sid";

/// An adaptive [`TokenStore`] that reads and writes session cookies securely
/// while supporting non-TLS development and local area network deployments.
#[derive(Debug, Clone)]
pub struct AdaptiveCookieTokenStore {
    name: Cow<'static, str>,
}

/// Context inputs for evaluating transport security.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TransportSecurityInputs<'a> {
    pub cookie_secure_env: Option<&'a str>,
    pub trust_forwarded_proto_env: Option<&'a str>,
    pub forwarded_proto_header: Option<&'a str>,
    pub uri_scheme: Option<&'a str>,
    pub public_base_url_env: Option<&'a str>,
    pub google_redirect_uri_env: Option<&'a str>,
}

impl AdaptiveCookieTokenStore {
    /// Creates a store using the default cookie name (`"sid"`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the base name of the session cookie.
    #[must_use]
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Returns the prefixed cookie name used for HTTPS (`__Host-<name>`).
    #[must_use]
    pub fn prefixed_name(&self) -> String {
        format!("__Host-{}", self.name)
    }

    /// Pure evaluator for whether transport should be treated as secure.
    ///
    /// Precedence:
    /// 1. `COOKIE_SECURE` ("true"/"1" -> true, "false"/"0" -> false).
    /// 2. `TRUST_FORWARDED_PROTO=true` AND `X-Forwarded-Proto: https` -> true.
    /// 3. URI scheme `https` -> true.
    /// 4. `PUBLIC_BASE_URL` starting with `https://` -> true.
    /// 5. `GOOGLE_REDIRECT_URI` starting with `https://` -> true.
    /// 6. Default -> false.
    #[must_use]
    pub fn is_secure(inputs: TransportSecurityInputs<'_>) -> bool {
        if let Some(val) = inputs.cookie_secure_env {
            let trimmed = val.trim();
            if trimmed.eq_ignore_ascii_case("true") || trimmed == "1" {
                return true;
            }
            if trimmed.eq_ignore_ascii_case("false") || trimmed == "0" {
                return false;
            }
        }

        let trust_proxy = inputs.trust_forwarded_proto_env.is_some_and(|v| {
            let t = v.trim();
            t.eq_ignore_ascii_case("true") || t == "1"
        });

        if trust_proxy
            && inputs
                .forwarded_proto_header
                .is_some_and(|proto| proto.eq_ignore_ascii_case("https"))
        {
            return true;
        }

        if inputs.uri_scheme == Some("https") {
            return true;
        }

        if inputs
            .public_base_url_env
            .is_some_and(|base| base.trim().starts_with("https://"))
        {
            return true;
        }

        if inputs
            .google_redirect_uri_env
            .is_some_and(|redirect| redirect.trim().starts_with("https://"))
        {
            return true;
        }

        false
    }

    /// Determines whether the current request or deployment environment is secure (HTTPS).
    #[must_use]
    pub fn is_secure_transport(&self, cx: &Cx) -> bool {
        let cookie_secure = std::env::var("COOKIE_SECURE").ok();
        let trust_forwarded = std::env::var("TRUST_FORWARDED_PROTO").ok();
        let parts = try_request_context::<http::request::Parts>(cx);
        let proto = parts
            .and_then(|p| p.headers.get("x-forwarded-proto"))
            .and_then(|v| v.to_str().ok());
        let uri_scheme = parts.and_then(|p| p.uri.scheme_str());
        let public_base_url = std::env::var("PUBLIC_BASE_URL").ok();
        let google_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").ok();

        Self::is_secure(TransportSecurityInputs {
            cookie_secure_env: cookie_secure.as_deref(),
            trust_forwarded_proto_env: trust_forwarded.as_deref(),
            forwarded_proto_header: proto,
            uri_scheme,
            public_base_url_env: public_base_url.as_deref(),
            google_redirect_uri_env: google_redirect_uri.as_deref(),
        })
    }
}

impl Default for AdaptiveCookieTokenStore {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(SESSION_COOKIE_NAME),
        }
    }
}

impl TokenStore for AdaptiveCookieTokenStore {
    fn read<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, Option<Token>> {
        Box::pin(async move {
            let jar = topcoat::cookie::cookies(cx);

            // 1. Check for __Host- prefixed cookie first (hardened RFC 6265bis)
            let host_name = self.prefixed_name();
            if let Some(token) = jar
                .get(&host_name)
                .and_then(|cookie| Token::decode(cookie.value_trimmed()).ok())
            {
                return Ok(Some(token));
            }

            // 2. On secure transport, reject plain/insecure cookie fallback to
            // prevent cookie tossing / session fixation from insecure origins or subdomains.
            if self.is_secure_transport(cx) {
                return Ok(None);
            }

            // 3. Check for plain cookie (non-TLS / dev)
            if let Some(token) = jar
                .get(&self.name)
                .and_then(|cookie| Token::decode(cookie.value_trimmed()).ok())
            {
                return Ok(Some(token));
            }

            Ok(None)
        })
    }

    fn write<'a>(
        &'a self,
        cx: &'a Cx,
        token: Token,
        max_age: Duration,
    ) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let max_age = topcoat::cookie::time::Duration::try_from(max_age)?;
            let is_secure = self.is_secure_transport(cx);

            let cookie_name = if is_secure {
                self.prefixed_name()
            } else {
                self.name.to_string()
            };

            let mut cookie = Cookie::new(cookie_name, token.encode());
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Lax);
            cookie.set_max_age(max_age);
            if is_secure {
                cookie.set_secure(true);
            }

            topcoat::cookie::cookies(cx).add(cookie);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let jar = topcoat::cookie::cookies(cx);

            // Clear prefixed cookie
            let mut host_cookie = Cookie::new(self.prefixed_name(), "");
            host_cookie.set_path("/");
            host_cookie.set_secure(true);
            jar.remove(host_cookie);

            // Clear plain cookie
            let mut plain_cookie = Cookie::new(self.name.to_string(), "");
            plain_cookie.set_path("/");
            jar.remove(plain_cookie);

            Ok(())
        })
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;
