//! `ETag` helpers for anonymous public offer-page caching.

const BUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Weak `ETag` comparison that ignores an optional `W/` prefix on either side.
#[must_use]
pub fn weak_etag_match(header_tag: &str, current_etag: &str) -> bool {
    let current_opaque = current_etag.strip_prefix("W/").unwrap_or(current_etag);
    let header_opaque = header_tag.strip_prefix("W/").unwrap_or(header_tag);
    current_opaque == header_opaque
}

/// Returns true when any tag in a comma-separated `If-None-Match` value matches.
#[must_use]
pub fn matches_if_none_match(inm_header: &str, current_etag: &str) -> bool {
    for raw_tag in inm_header.split(',') {
        let tag = raw_tag.trim();
        if tag.is_empty() {
            continue;
        }
        if tag == "*" || weak_etag_match(tag, current_etag) {
            return true;
        }
    }
    false
}

/// Builds the weak-comparable `ETag` for a public offer detail response.
#[must_use]
pub fn offer_etag(offer_id: u128, updated_at_micros: i64, maker_updated_at_micros: i64) -> String {
    format!("\"{offer_id:x}-{updated_at_micros:x}-{maker_updated_at_micros:x}-v{BUILD_VERSION}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_etag_match() {
        assert!(weak_etag_match("\"tag\"", "\"tag\""));
        assert!(weak_etag_match("W/\"tag\"", "\"tag\""));
        assert!(weak_etag_match("\"tag\"", "W/\"tag\""));
        assert!(weak_etag_match("W/\"tag\"", "W/\"tag\""));
        assert!(!weak_etag_match("\"other\"", "\"tag\""));
    }

    #[test]
    fn test_matches_if_none_match() {
        let current = "\"123-456-v0.1.0\"";
        assert!(matches_if_none_match("*", current));
        assert!(matches_if_none_match("\"123-456-v0.1.0\"", current));
        assert!(matches_if_none_match("W/\"123-456-v0.1.0\"", current));
        assert!(matches_if_none_match(
            "\"other\", W/\"123-456-v0.1.0\"",
            current
        ));
        assert!(matches_if_none_match(
            " \"other\" ,  \"123-456-v0.1.0\" ",
            current
        ));
        assert!(!matches_if_none_match("\"other\", \"second\"", current));
        assert!(!matches_if_none_match("", current));
        assert!(!matches_if_none_match(" , ", current));
    }

    #[test]
    fn offer_etag_includes_build_version() {
        let tag = offer_etag(1, 2, 3);
        assert!(tag.contains(BUILD_VERSION));
        assert!(tag.starts_with('"'));
        assert!(tag.ends_with('"'));
    }

    #[test]
    fn offer_etag_changes_when_maker_changes() {
        assert_ne!(offer_etag(1, 2, 3), offer_etag(1, 2, 4));
    }
}
