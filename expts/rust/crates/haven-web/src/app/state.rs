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
    pub topcoat_dev_url: Option<&'a str>,
}

/// Pure decision function computing whether email delivery should be active.
///
/// If `EMAIL_DELIVERY_ENABLED` is explicitly configured, its boolean value takes precedence.
/// Otherwise, email delivery defaults to enabled ONLY when an environment marker explicitly
/// identifies development (`APP_ENV`, `ENVIRONMENT`, or `ENV` set to `"development"` or `"dev"`,
/// or when `TOPCOAT_DEV_URL` is set and non-empty).
///
/// In production, unrecognized, or unset environments, email delivery defaults to disabled
/// to ensure production safety and prevent unconfigured email delivery attempts.
#[must_use]
pub fn compute_email_delivery_enabled(inputs: EmailDeliveryInputs<'_>) -> bool {
    if let Some(explicit) = inputs.email_delivery_enabled {
        let trimmed = explicit.trim();
        return trimmed.eq_ignore_ascii_case("true") || trimmed == "1";
    }

    let is_dev = |val: Option<&str>| {
        val.is_some_and(|v| {
            let t = v.trim();
            t.eq_ignore_ascii_case("development") || t.eq_ignore_ascii_case("dev")
        })
    };

    let has_topcoat_dev = inputs.topcoat_dev_url.is_some_and(|v| !v.trim().is_empty());

    is_dev(inputs.app_env) || is_dev(inputs.environment) || is_dev(inputs.env) || has_topcoat_dev
}

/// Returns whether email delivery infrastructure is active based on environment variables.
///
/// In production, unset, or unrecognized environments, defaults to `false` unless explicitly activated
/// via `EMAIL_DELIVERY_ENABLED=true`. In explicit development mode, defaults to `true` to enable
/// local file-based mail delivery.
#[must_use]
pub fn is_email_delivery_enabled() -> bool {
    let email_delivery = std::env::var("EMAIL_DELIVERY_ENABLED").ok();
    let app_env = std::env::var("APP_ENV").ok();
    let environment = std::env::var("ENVIRONMENT").ok();
    let env = std::env::var("ENV").ok();
    let topcoat_dev_url = std::env::var("TOPCOAT_DEV_URL").ok();

    compute_email_delivery_enabled(EmailDeliveryInputs {
        email_delivery_enabled: email_delivery.as_deref(),
        app_env: app_env.as_deref(),
        environment: environment.as_deref(),
        env: env.as_deref(),
        topcoat_dev_url: topcoat_dev_url.as_deref(),
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
    fn test_compute_email_delivery_enabled_defaults_to_false_when_unset_or_production() {
        // Defaults to false (production safe) when unset or empty
        assert!(!compute_email_delivery_enabled(
            EmailDeliveryInputs::default()
        ));
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            app_env: Some(""),
            ..Default::default()
        }));

        // Production or unrecognized environments default to false
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
        assert!(!compute_email_delivery_enabled(EmailDeliveryInputs {
            app_env: Some("staging"),
            ..Default::default()
        }));
    }

    #[test]
    fn test_compute_email_delivery_enabled_defaults_to_true_for_explicit_dev() {
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            app_env: Some("development"),
            ..Default::default()
        }));
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            environment: Some("dev"),
            ..Default::default()
        }));
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            env: Some("DEV"),
            ..Default::default()
        }));
        assert!(compute_email_delivery_enabled(EmailDeliveryInputs {
            topcoat_dev_url: Some("http://localhost:8080"),
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
