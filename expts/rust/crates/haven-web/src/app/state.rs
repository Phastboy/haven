//! Application state and context accessors.

use haven_domain::ports::{Registry, RepoError};
use topcoat::{
    Error as RouterError,
    context::{Cx, app_context},
};

#[derive(Clone)]
pub struct AppState {
    pub registry: std::sync::Arc<dyn Registry>,
    pub google_oauth: Option<crate::app::auth::google::GoogleOAuthConfig>,
    pub http_client: reqwest::Client,
}

pub fn registry(cx: &Cx) -> &dyn Registry {
    &*app_context::<AppState>(cx).registry
}

pub fn google_oauth(cx: &Cx) -> Option<&crate::app::auth::google::GoogleOAuthConfig> {
    app_context::<AppState>(cx).google_oauth.as_ref()
}

pub fn http_client(cx: &Cx) -> &reqwest::Client {
    &app_context::<AppState>(cx).http_client
}

pub fn map_repo_err(e: RepoError) -> RouterError {
    match e {
        RepoError::NotFound { .. } => topcoat::router::error::not_found().into(),
        RepoError::Conflict { entity, key } => {
            topcoat::router::error::bad_request(format!("Conflict on {entity}: {key}")).into()
        }
        RepoError::Corrupt(msg) | RepoError::Unavailable(msg) => topcoat::Error::msg(msg),
    }
}

/// Returns whether email delivery infrastructure is active.
///
/// While email transport infrastructure is unconfigured, this returns `false`
/// so that users are never misled into expecting a magic link email that cannot arrive.
/// When mail delivery infrastructure is ready, setting `EMAIL_DELIVERY_ENABLED=true`
/// immediately activates full email authentication.
pub fn is_email_delivery_enabled() -> bool {
    std::env::var("EMAIL_DELIVERY_ENABLED")
        .is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1")
}
