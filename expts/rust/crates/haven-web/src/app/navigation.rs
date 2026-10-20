//! Navigation layout rules and safe back-navigation resolution.

use http::HeaderMap;

/// Layout mode for navigation chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationLayout {
    /// Top-level route (e.g. `/offers`, `/profile`): renders primary navigation on all screen sizes.
    Main,
    /// Child route (e.g. `/offers/{slug}`, `/profile/edit`): renders back header only on mobile,
    /// and primary navigation + back header on large screens.
    Child {
        /// Fallback parent route when no valid same-site referer is available.
        fallback_parent: &'static str,
    },
}

/// Classifies a route path into either [`NavigationLayout::Main`] or [`NavigationLayout::Child`].
///
/// When `current_user_handle` matches the handle in `/profiles/{handle}`, the fallback parent is
/// set to `/profile` (the owner's profile shell).
#[must_use]
pub fn classify_route(path: &str, current_user_handle: Option<&str>) -> NavigationLayout {
    let normalized = path.trim_end_matches('/');
    let clean_path = if normalized.is_empty() {
        "/"
    } else {
        normalized
    };

    match clean_path {
        "/" | "/offers" | "/profile" | "/conversations" => NavigationLayout::Main,
        p if p.starts_with("/conversations/") => NavigationLayout::Child {
            fallback_parent: "/conversations",
        },
        "/profile/offers/new" => NavigationLayout::Child {
            fallback_parent: "/profile/offers",
        },
        p if p.starts_with("/profile/offers/") && p.ends_with("/edit") => NavigationLayout::Child {
            fallback_parent: "/profile/offers",
        },
        p if p.starts_with("/profile/offers/") => NavigationLayout::Child {
            fallback_parent: "/profile/offers",
        },
        "/profile/offers" => NavigationLayout::Child {
            fallback_parent: "/profile",
        },
        p if p.starts_with("/profile/") => NavigationLayout::Child {
            fallback_parent: "/profile",
        },
        p if p.starts_with("/offers/") => NavigationLayout::Child {
            fallback_parent: "/offers",
        },
        p if p.starts_with("/profiles/") => {
            let target_handle = p.strip_prefix("/profiles/").unwrap_or("");
            let fallback_parent = if let Some(own_handle) = current_user_handle {
                if target_handle == own_handle {
                    "/profile"
                } else {
                    "/offers"
                }
            } else {
                "/offers"
            };
            NavigationLayout::Child { fallback_parent }
        }
        p if p.starts_with("/auth/") => NavigationLayout::Child {
            fallback_parent: "/",
        },
        _ => NavigationLayout::Child {
            fallback_parent: "/offers",
        },
    }
}

/// Determines if navigating back to `referer_path` from `current_path` would cause
/// a navigation loop or cross-hierarchy regression.
///
/// Returns `true` if `referer_path` should be rejected in favor of `fallback_parent`:
/// 1. Self-referential loop (e.g. page refresh): `referer_path == current_path`.
/// 2. Descendant child loop: `referer_path` is a child of `current_path`
///    (e.g. ascending from `/profile/offers/{id}` to `/profile/offers`).
/// 3. Cross-lane owner/public loop:
///    When on an owner management page (`/profile/...`), reject referers from the public
///    discovery lane (`/offers/...` or `/profiles/...`). When an owner views the public
///    offer page and clicks back to return to the owner management view (`/profile/offers/{id}`),
///    the owner view's back button must proceed up to `/profile/offers`, never looping
///    back and forth with `/offers/{slug}`.
fn should_reject_referer(referer_path: &str, current_path: &str) -> bool {
    let ref_clean = referer_path.trim_end_matches('/');
    let curr_clean = current_path.trim_end_matches('/');

    // 1. Self loop (page refresh)
    if ref_clean == curr_clean {
        return true;
    }

    // 2. Descendant child loop (ascending up from a child page)
    if !curr_clean.is_empty() && curr_clean != "/" {
        let child_prefix = format!("{curr_clean}/");
        if ref_clean.starts_with(&child_prefix) {
            return true;
        }
    }

    // 3. Cross-lane owner/public loop:
    // Owner pages (/profile/...) must never point back to public views (/offers/... or /profiles/...).
    if (curr_clean.starts_with("/profile/") || curr_clean == "/profile")
        && (ref_clean.starts_with("/offers/") || ref_clean.starts_with("/profiles/"))
    {
        return true;
    }

    false
}

fn extract_host_name(h: &str) -> &str {
    let h = h.trim();
    if let Some((ipv6, _)) = h.strip_prefix('[').and_then(|rest| rest.split_once(']')) {
        return ipv6.trim();
    }
    match h.split_once(':') {
        Some((name, _)) => name.trim(),
        None => h,
    }
}

/// Checks whether a referer URL belongs to the same site as the incoming request host.
fn is_same_site(expected_host: &str, referer_url: &url::Url) -> bool {
    let expected_name = extract_host_name(expected_host);
    let referer_raw = referer_url.host_str().unwrap_or("").trim();
    let referer_name = extract_host_name(referer_raw);

    let is_local = |h: &str| {
        h.eq_ignore_ascii_case("localhost") || h == "127.0.0.1" || h == "::1" || h == "[::1]"
    };

    // Both are local loopback dev hosts
    if is_local(expected_name) && is_local(referer_name) {
        return true;
    }

    // Exact host match
    if expected_name.eq_ignore_ascii_case(referer_name) {
        return true;
    }

    // nip.io local network domain match (e.g. 192.168.0.50.nip.io)
    if expected_name.ends_with(".nip.io") && referer_name.ends_with(".nip.io") {
        return true;
    }

    false
}

/// Resolves a safe same-site return URL from the `Referer` request header.
///
/// To prevent open redirect and UI loop vulnerabilities:
/// - If `Referer` is absent, invalid, or cross-origin: returns `fallback_parent`.
/// - If `Referer` points to `current_path` or a descendant child (e.g. page refresh or returning up from a child): returns `fallback_parent`.
/// - Protocol-relative (`//`), `javascript:`, and non-HTTP URLs are strictly rejected.
/// - Only same-site relative paths (with optional query parameters) are accepted.
#[must_use]
pub fn resolve_safe_back_url(
    headers: &HeaderMap,
    current_path: &str,
    fallback_parent: &str,
) -> String {
    let Some(referer_val) = headers.get(http::header::REFERER) else {
        return fallback_parent.to_string();
    };

    let Ok(referer_str) = referer_val.to_str() else {
        return fallback_parent.to_string();
    };

    let referer = referer_str.trim();
    if referer.is_empty() {
        return fallback_parent.to_string();
    }

    // Determine the host of the current request: prioritize X-Forwarded-Host when behind proxies
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(http::header::HOST))
        .and_then(|h| h.to_str().ok())
        .map(|h| h.split(',').next().unwrap_or(h).trim());

    // If referer is an absolute URL (e.g. http://localhost:3000/profile)
    if referer.starts_with("http://") || referer.starts_with("https://") {
        let Ok(parsed_url) = url::Url::parse(referer) else {
            return fallback_parent.to_string();
        };

        // Host verification: must match current host/origin to be considered same-site
        if host.is_some_and(|expected_host| !is_same_site(expected_host, &parsed_url)) {
            return fallback_parent.to_string();
        }

        let path = parsed_url.path();
        // Prevent loop when referer matches current path, is a descendant, or causes cross-lane cycles
        if should_reject_referer(path, current_path) {
            return fallback_parent.to_string();
        }

        // Return path with query if present
        return match parsed_url.query() {
            Some(query) => format!("{path}?{query}"),
            None => path.to_string(),
        };
    }

    // Reject protocol-relative URLs (`//evil.com`), backslashes, or non-slash prefixes
    if referer.starts_with("//") || referer.starts_with("/\\") || !referer.starts_with('/') {
        return fallback_parent.to_string();
    }

    // Extract path portion without query for loop checking
    let path_part = referer.split('?').next().unwrap_or(referer);
    if should_reject_referer(path_part, current_path) {
        return fallback_parent.to_string();
    }

    referer.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::{HeaderName, HeaderValue};

    #[test]
    fn test_classify_route() {
        assert_eq!(classify_route("/", None), NavigationLayout::Main);
        assert_eq!(classify_route("/offers", None), NavigationLayout::Main);
        assert_eq!(classify_route("/offers/", None), NavigationLayout::Main);
        assert_eq!(classify_route("/profile", None), NavigationLayout::Main);
        assert_eq!(classify_route("/profile/", None), NavigationLayout::Main);
        assert_eq!(
            classify_route("/conversations", None),
            NavigationLayout::Main
        );
        assert_eq!(
            classify_route("/conversations/", None),
            NavigationLayout::Main
        );

        // Child routes
        assert_eq!(
            classify_route("/conversations/3f2504e0-4f89-11d3-9a0c-0305e82c3301", None),
            NavigationLayout::Child {
                fallback_parent: "/conversations"
            }
        );
        assert_eq!(
            classify_route("/offers/rustic-table-123", None),
            NavigationLayout::Child {
                fallback_parent: "/offers"
            }
        );
        assert_eq!(
            classify_route("/profiles/alex-wood", None),
            NavigationLayout::Child {
                fallback_parent: "/offers"
            }
        );
        assert_eq!(
            classify_route("/profiles/alex-wood", Some("other-maker")),
            NavigationLayout::Child {
                fallback_parent: "/offers"
            }
        );
        assert_eq!(
            classify_route("/profiles/alex-wood", Some("alex-wood")),
            NavigationLayout::Child {
                fallback_parent: "/profile"
            }
        );
        assert_eq!(
            classify_route("/profile/edit", None),
            NavigationLayout::Child {
                fallback_parent: "/profile"
            }
        );
        assert_eq!(
            classify_route("/profile/offers", None),
            NavigationLayout::Child {
                fallback_parent: "/profile"
            }
        );
        assert_eq!(
            classify_route("/profile/offers/new", None),
            NavigationLayout::Child {
                fallback_parent: "/profile/offers"
            }
        );
        assert_eq!(
            classify_route("/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301", None),
            NavigationLayout::Child {
                fallback_parent: "/profile/offers"
            }
        );
        assert_eq!(
            classify_route(
                "/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301/edit",
                None
            ),
            NavigationLayout::Child {
                fallback_parent: "/profile/offers"
            }
        );
        assert_eq!(
            classify_route("/profile/account", None),
            NavigationLayout::Child {
                fallback_parent: "/profile"
            }
        );
        assert_eq!(
            classify_route("/auth/sign-in", None),
            NavigationLayout::Child {
                fallback_parent: "/"
            }
        );
    }

    #[test]
    fn test_resolve_safe_back_url_when_missing() {
        let headers = HeaderMap::new();
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/offers");
        assert_eq!(back, "/offers");
    }

    #[test]
    fn test_resolve_safe_back_url_same_site_relative() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::REFERER, HeaderValue::from_static("/offers"));
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/fallback");
        assert_eq!(back, "/offers");
    }

    #[test]
    fn test_resolve_safe_back_url_same_site_full_url() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("localhost:3000"),
        );
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("http://localhost:3000/profiles/alex?after=123"),
        );
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/offers");
        assert_eq!(back, "/profiles/alex?after=123");
    }

    #[test]
    fn test_resolve_safe_back_url_rejects_cross_site() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("localhost:3000"),
        );
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("https://evil.example.com/phishing"),
        );
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/offers");
        assert_eq!(back, "/offers");
    }

    #[test]
    fn test_resolve_safe_back_url_rejects_protocol_relative() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("//evil.example.com/login"),
        );
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/offers");
        assert_eq!(back, "/offers");
    }

    #[test]
    fn test_resolve_safe_back_url_rejects_self_loop_on_refresh() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("/offers/abc"),
        );
        let back = resolve_safe_back_url(&headers, "/offers/abc", "/offers");
        assert_eq!(back, "/offers");
    }

    #[test]
    fn test_resolve_safe_back_url_rejects_child_descendant_loop() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("localhost:3000"),
        );
        // Coming back up from /profile/offers/{id} to /profile/offers must NOT link back down to {id}
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static(
                "http://localhost:3000/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            ),
        );
        let back = resolve_safe_back_url(&headers, "/profile/offers", "/profile");
        assert_eq!(back, "/profile");

        // Relative referer for child route also rejected
        let mut headers_rel = HeaderMap::new();
        headers_rel.insert(
            http::header::REFERER,
            HeaderValue::from_static("/profile/offers/new"),
        );
        let back_rel = resolve_safe_back_url(&headers_rel, "/profile/offers", "/profile");
        assert_eq!(back_rel, "/profile");
    }

    #[test]
    fn test_resolve_safe_back_url_from_owner_offer_to_public_view() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("localhost:3000"),
        );
        // Navigating to public offer from owner's command center must return to owner's command center
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static(
                "http://localhost:3000/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            ),
        );
        let back = resolve_safe_back_url(&headers, "/offers/rustic-table-123", "/offers");
        assert_eq!(back, "/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301");
    }

    #[test]
    fn test_resolve_safe_back_url_with_reverse_proxy_and_nip_io() {
        let mut headers = HeaderMap::new();
        // Server socket sees internal host:port, but reverse proxy sets X-Forwarded-Host
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("127.0.0.1:3000"),
        );
        headers.insert(
            HeaderName::from_static("x-forwarded-host"),
            HeaderValue::from_static("192.168.0.50.nip.io:8080"),
        );
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static(
                "http://192.168.0.50.nip.io:8080/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            ),
        );
        let back = resolve_safe_back_url(&headers, "/offers/rustic-table-123", "/offers");
        assert_eq!(back, "/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301");
    }

    #[test]
    fn test_resolve_safe_back_url_prevents_loop_between_owner_offer_and_public_view() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::HOST,
            HeaderValue::from_static("localhost:3000"),
        );
        // Returning from public offer page to owner management view must NOT link back to public view
        headers.insert(
            http::header::REFERER,
            HeaderValue::from_static("http://localhost:3000/offers/rustic-table-123"),
        );
        let back = resolve_safe_back_url(
            &headers,
            "/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "/profile/offers",
        );
        assert_eq!(back, "/profile/offers");
    }
}
