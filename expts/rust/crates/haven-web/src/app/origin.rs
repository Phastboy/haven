//! Origin and CSRF policy configuration for the web server.

use topcoat::router::OriginPolicy;

/// Pure function computing the set of trusted origins from environment inputs.
#[must_use]
pub fn compute_trusted_origins(
    port: &str,
    public_base_url: Option<&str>,
    trusted_origins_env: Option<&str>,
) -> Vec<String> {
    let mut origins = Vec::new();

    // Standard local development origins for the configured port
    origins.push(format!("http://localhost:{port}"));
    origins.push(format!("http://127.0.0.1:{port}"));
    origins.push(format!("http://[::1]:{port}"));
    origins.push(format!("https://localhost:{port}"));
    origins.push(format!("https://127.0.0.1:{port}"));
    origins.push(format!("https://[::1]:{port}"));

    // Standard local development origins on default port 3000
    if port != "3000" {
        origins.push("http://localhost:3000".to_string());
        origins.push("http://127.0.0.1:3000".to_string());
        origins.push("https://localhost:3000".to_string());
        origins.push("https://127.0.0.1:3000".to_string());
    }

    if let Some(base_url) = public_base_url {
        let trimmed = base_url.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                origins.push(trimmed.to_string());
            } else {
                origins.push(format!("http://{trimmed}"));
                origins.push(format!("https://{trimmed}"));
            }

            // If PUBLIC_BASE_URL contains an IP with nip.io (e.g. 192.168.0.50.nip.io:8080),
            // also support direct IP access on that port and server port.
            if let Some((ip_part, _)) = trimmed.split_once(".nip.io") {
                let ip_clean = ip_part
                    .trim_start_matches("http://")
                    .trim_start_matches("https://");
                if !ip_clean.is_empty() {
                    origins.push(format!("http://{ip_clean}:{port}"));
                    origins.push(format!("https://{ip_clean}:{port}"));
                    if let Some((_, nip_port)) = trimmed.rsplit_once(':') {
                        origins.push(format!("http://{ip_clean}:{nip_port}"));
                        origins.push(format!("https://{ip_clean}:{nip_port}"));
                    }
                }
            }
        }
    }

    if let Some(extra) = trusted_origins_env {
        for entry in extra.split(',') {
            let trimmed = entry.trim().trim_end_matches('/');
            if !trimmed.is_empty() {
                origins.push(trimmed.to_string());
            }
        }
    }

    origins.sort();
    origins.dedup();
    origins
}

/// Inputs used to determine whether origin verification should be disabled in development mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct OriginPolicyInputs<'a> {
    pub app_env: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub env: Option<&'a str>,
    pub topcoat_dev_url: Option<&'a str>,
    pub strict_origin_policy: Option<&'a str>,
}

/// Determines whether origin verification should be disabled in development mode.
///
/// Origin verification is disabled ONLY when an environment marker explicitly identifies
/// development (`APP_ENV`, `ENVIRONMENT`, or `ENV` set to `"development"` or `"dev"`,
/// or when `TOPCOAT_DEV_URL` is set and non-empty), provided `STRICT_ORIGIN_POLICY` is not set.
/// Unset, empty, unrecognized, or production environments default to production-safe behavior (`false`).
#[must_use]
pub fn should_disable_origin_verification(inputs: OriginPolicyInputs<'_>) -> bool {
    let enforce_strict = inputs
        .strict_origin_policy
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("true") || v.trim() == "1");

    if enforce_strict {
        return false;
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

/// Constructs the application [`OriginPolicy`] configured with trusted origins.
///
/// In local development (when explicitly identified by `APP_ENV`, `ENVIRONMENT`, or `ENV` set
/// to `"development"` or `"dev"`, or when `TOPCOAT_DEV_URL` is set), origin verification is
/// disabled using [`OriginPolicy::dangerous_disable()`] so that developers and paired testing devices
/// on the local network (accessing `0.0.0.0:8080` via LAN IP or Wi-Fi) are not blocked by browser
/// Fetch-Metadata (`Sec-Fetch-Site: cross-site`) or cross-IP origin restrictions.
/// Setting `STRICT_ORIGIN_POLICY=true` allows forcing strict verification in local testing if desired.
///
/// In production environments, or whenever environment markers are unset or unrecognized,
/// strict origin verification is enforced against trusted origins derived from `PORT`,
/// `PUBLIC_BASE_URL`, and `TRUSTED_ORIGINS`.
#[must_use]
pub fn build_origin_policy() -> OriginPolicy {
    let app_env = std::env::var("APP_ENV").ok();
    let environment = std::env::var("ENVIRONMENT").ok();
    let env = std::env::var("ENV").ok();
    let topcoat_dev_url = std::env::var("TOPCOAT_DEV_URL").ok();
    let strict_origin_policy = std::env::var("STRICT_ORIGIN_POLICY").ok();

    if should_disable_origin_verification(OriginPolicyInputs {
        app_env: app_env.as_deref(),
        environment: environment.as_deref(),
        env: env.as_deref(),
        topcoat_dev_url: topcoat_dev_url.as_deref(),
        strict_origin_policy: strict_origin_policy.as_deref(),
    }) {
        return OriginPolicy::dangerous_disable();
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let public_base_url = std::env::var("PUBLIC_BASE_URL").ok();
    let trusted_origins_env = std::env::var("TRUSTED_ORIGINS").ok();

    let origins = compute_trusted_origins(
        &port,
        public_base_url.as_deref(),
        trusted_origins_env.as_deref(),
    );

    OriginPolicy::new().trust_origins(origins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_trusted_origins_defaults() {
        let origins = compute_trusted_origins("3000", None, None);
        assert!(origins.contains(&"http://localhost:3000".to_string()));
        assert!(origins.contains(&"http://127.0.0.1:3000".to_string()));
        assert!(origins.contains(&"http://[::1]:3000".to_string()));
    }

    #[test]
    fn test_compute_trusted_origins_with_public_base_url_and_nip_io() {
        let origins = compute_trusted_origins(
            "3000",
            Some("192.168.0.50.nip.io:8080"),
            Some("http://custom.dev:3000, https://app.example.com"),
        );
        assert!(origins.contains(&"http://localhost:3000".to_string()));
        assert!(origins.contains(&"http://192.168.0.50.nip.io:8080".to_string()));
        assert!(origins.contains(&"https://192.168.0.50.nip.io:8080".to_string()));
        assert!(origins.contains(&"http://192.168.0.50:3000".to_string()));
        assert!(origins.contains(&"http://192.168.0.50:8080".to_string()));
        assert!(origins.contains(&"http://custom.dev:3000".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
    }

    #[test]
    fn test_should_disable_origin_verification_behavior() {
        // Defaults to false (production safe) when unset or empty
        assert!(!should_disable_origin_verification(
            OriginPolicyInputs::default()
        ));
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            app_env: Some(""),
            ..Default::default()
        }));

        // Production or unrecognized environments remain production safe (false)
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            app_env: Some("production"),
            ..Default::default()
        }));
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            app_env: Some("staging"),
            ..Default::default()
        }));
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            environment: Some("prod"),
            ..Default::default()
        }));

        // Explicit development markers disable verification (true)
        assert!(should_disable_origin_verification(OriginPolicyInputs {
            app_env: Some("development"),
            ..Default::default()
        }));
        assert!(should_disable_origin_verification(OriginPolicyInputs {
            environment: Some("dev"),
            ..Default::default()
        }));
        assert!(should_disable_origin_verification(OriginPolicyInputs {
            env: Some("DEV"),
            ..Default::default()
        }));
        assert!(should_disable_origin_verification(OriginPolicyInputs {
            topcoat_dev_url: Some("http://localhost:8080"),
            ..Default::default()
        }));

        // Strict origin override keeps verification enabled even in development
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            app_env: Some("development"),
            strict_origin_policy: Some("true"),
            ..Default::default()
        }));
        assert!(!should_disable_origin_verification(OriginPolicyInputs {
            environment: Some("dev"),
            strict_origin_policy: Some("1"),
            ..Default::default()
        }));
    }
}
