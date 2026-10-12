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
    let clean_path = if normalized.is_empty() { "/" } else { normalized };

    match clean_path {
        "/" | "/offers" | "/profile" => NavigationLayout::Main,
        "/profile/offers/new" => {
            NavigationLayout::Child {
                fallback_parent: "/profile/offers",
            }
        }
        p if p.starts_with("/profile/offers/") && p.ends_with("/edit") => {
            NavigationLayout::Child {
                fallback_parent: "/profile/offers",
            }
        }
        p if p.starts_with("/profile/offers/") => {
            NavigationLayout::Child {
                fallback_parent: "/profile/offers",
            }
        }
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

/// Resolves a safe same-site return URL from the `Referer` request header.
///
/// To prevent open redirect and UI loop vulnerabilities:
/// - If `Referer` is absent, invalid, or cross-origin: returns `fallback_parent`.
/// - If `Referer` points to `current_path` (e.g. page refresh): returns `fallback_parent`.
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

    // Determine the host of the current request if available
    let host = headers
        .get(http::header::HOST)
        .or_else(|| headers.get("x-forwarded-host"))
        .and_then(|h| h.to_str().ok())
        .map(str::trim);

    // If referer is an absolute URL (e.g. http://localhost:3000/profile)
    if referer.starts_with("http://") || referer.starts_with("https://") {
        let Ok(parsed_url) = url::Url::parse(referer) else {
            return fallback_parent.to_string();
        };

        // Host verification: must match current Host header to be considered same-site
        if let Some(expected_host) = host {
            let (expected_name, expected_port) = match expected_host.split_once(':') {
                Some((name, port)) => (name, port.parse::<u16>().ok()),
                None => (expected_host, None),
            };

            let referer_name = parsed_url.host_str().unwrap_or("");
            let referer_port = parsed_url.port();

            let names_match = expected_name.eq_ignore_ascii_case(referer_name);
            let ports_match = match (expected_port, referer_port) {
                (Some(p1), Some(p2)) => p1 == p2,
                (Some(p), None) | (None, Some(p)) => {
                    (parsed_url.scheme() == "http" && p == 80)
                        || (parsed_url.scheme() == "https" && p == 443)
                }
                (None, None) => true,
            };

            if !names_match || !ports_match {
                return fallback_parent.to_string();
            }
        }

        let path = parsed_url.path();
        // Prevent loop when referer matches current path
        if path == current_path {
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
    if path_part == current_path {
        return fallback_parent.to_string();
    }

    referer.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_classify_route() {
        assert_eq!(classify_route("/", None), NavigationLayout::Main);
        assert_eq!(classify_route("/offers", None), NavigationLayout::Main);
        assert_eq!(classify_route("/offers/", None), NavigationLayout::Main);
        assert_eq!(classify_route("/profile", None), NavigationLayout::Main);
        assert_eq!(classify_route("/profile/", None), NavigationLayout::Main);

        // Child routes
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
            classify_route("/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301/edit", None),
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
}
