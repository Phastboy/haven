//! Google OAuth 2.0 configuration, client, and route assembly.

pub mod client;
pub mod config;
pub mod routes;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    reason = "test assertions and mocks"
)]
mod tests;

pub use config::GoogleOAuthConfig;
#[cfg(test)]
pub use config::OAuthConfigError;

#[cfg(test)]
pub use client::{GoogleTokenResponse, GoogleUserInfo};
#[cfg(test)]
pub use routes::{CallbackQuery, OAUTH_STATE_COOKIE_NAME};
