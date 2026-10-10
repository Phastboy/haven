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
    pub email_delivery_enabled: Option<bool>,
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
        RepoError::Conflict { entity, .. } => {
            topcoat::router::error::bad_request(format!("Conflict on {entity}")).into()
        }
        RepoError::Corrupt(msg) | RepoError::Unavailable(msg) => topcoat::Error::msg(msg),
    }
}

/// Inputs for evaluating whether email delivery is active.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmailDeliveryInputs<'a> {
    pub email_delivery_enabled: Option<&'a str>,
    pub app_env: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub env: Option<&'a str>,
}

/// Pure decision function computing whether email delivery should be active.
///
/// In production environments (`APP_ENV`, `ENVIRONMENT`, or `ENV` set to `"production"` or `"prod"`),
/// email delivery defaults to disabled unless explicitly activated via `EMAIL_DELIVERY_ENABLED=true` (or `1`).
///
/// In development and non-production environments, email delivery defaults to enabled
/// to allow local file-based mail delivery (`target/mail`). It can still be explicitly
/// disabled by setting `EMAIL_DELIVERY_ENABLED=false` (or `0`).
#[must_use]
pub fn compute_email_delivery_enabled(inputs: EmailDeliveryInputs<'_>) -> bool {
    if let Some(explicit) = inputs.email_delivery_enabled {
        let trimmed = explicit.trim();
        return trimmed.eq_ignore_ascii_case("true") || trimmed == "1";
    }

    let is_prod = |val: Option<&str>| {
        val.is_some_and(|v| {
            let t = v.trim();
            t.eq_ignore_ascii_case("production") || t.eq_ignore_ascii_case("prod")
        })
    };

    let in_production =
        is_prod(inputs.app_env) || is_prod(inputs.environment) || is_prod(inputs.env);

    !in_production
}

/// Returns whether email delivery infrastructure is active based on environment variables.
///
/// In production, defaults to `false` unless explicitly activated via `EMAIL_DELIVERY_ENABLED=true`.
/// In development and non-production modes, defaults to `true` to enable file-based mail delivery.
#[must_use]
pub fn is_email_delivery_enabled() -> bool {
    let email_delivery = std::env::var("EMAIL_DELIVERY_ENABLED").ok();
    let app_env = std::env::var("APP_ENV").ok();
    let environment = std::env::var("ENVIRONMENT").ok();
    let env = std::env::var("ENV").ok();

    compute_email_delivery_enabled(EmailDeliveryInputs {
        email_delivery_enabled: email_delivery.as_deref(),
        app_env: app_env.as_deref(),
        environment: environment.as_deref(),
        env: env.as_deref(),
    })
}

/// Returns whether email delivery is active for the current application context.
#[must_use]
pub fn email_delivery_enabled(cx: &Cx) -> bool {
    app_context::<AppState>(cx)
        .email_delivery_enabled
        .unwrap_or_else(is_email_delivery_enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_email_delivery_enabled_defaults_to_true_in_dev() {
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs::default()));
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            app_env: Some("development"),
            ..Default::default()
        }));
    }

    #[test]
    fn test_compute_email_delivery_enabled_defaults_to_false_in_production() {
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            app_env: Some("production"),
            ..Default::default()
        }));
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            environment: Some("prod"),
            ..Default::default()
        }));
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            env: Some("production"),
            ..Default::default()
        }));
    }

    #[test]
    fn test_compute_email_delivery_enabled_explicit_overrides() {
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            email_delivery_enabled: Some("true"),
            app_env: Some("production"),
            ..Default::default()
        }));
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            email_delivery_enabled: Some("1"),
            environment: Some("production"),
            ..Default::default()
        }));
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            email_delivery_enabled: Some("false"),
            app_env: Some("development"),
            ..Default::default()
        }));
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            email_delivery_enabled: Some("0"),
            ..Default::default()
        }));
    }
}
