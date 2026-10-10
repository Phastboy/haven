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

/// Constructs the application [`OriginPolicy`] configured with trusted origins.
///
/// In production environments (`APP_ENV=production`, `ENVIRONMENT=production`, or `ENV=production`),
/// strict origin verification is enforced against trusted origins derived from `PORT`, `PUBLIC_BASE_URL`,
/// and `TRUSTED_ORIGINS`.
///
/// In local development (when not running in production, or when `TOPCOAT_DEV_URL` is set),
/// origin verification is disabled using [`OriginPolicy::dangerous_disable()`] so that developers
/// and paired testing devices on the local network (accessing `0.0.0.0:8080` via LAN IP or Wi-Fi)
/// are not blocked by browser Fetch-Metadata (`Sec-Fetch-Site: cross-site`) or cross-IP origin restrictions.
/// Setting `STRICT_ORIGIN_POLICY=true` allows forcing strict verification in local testing if desired.
#[must_use]
pub fn build_origin_policy() -> OriginPolicy {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let public_base_url = std::env::var("PUBLIC_BASE_URL").ok();
    let trusted_origins_env = std::env::var("TRUSTED_ORIGINS").ok();
    let enforce_strict = std::env::var("STRICT_ORIGIN_POLICY")
        .is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1");

    let is_prod = |var: &str| {
        std::env::var(var).is_ok_and(|v| {
            let t = v.trim();
            t.eq_ignore_ascii_case("production") || t.eq_ignore_ascii_case("prod")
        })
    };
    let in_production = is_prod("APP_ENV") || is_prod("ENVIRONMENT") || is_prod("ENV");

    if !in_production && !enforce_strict {
        return OriginPolicy::dangerous_disable();
    }

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
}
