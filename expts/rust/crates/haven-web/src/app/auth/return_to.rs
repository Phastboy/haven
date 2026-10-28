//! Short-lived, validated feed intent across Google or magic-link sign-in.

use topcoat::{
    context::Cx,
    cookie::{Cookie, Cookies, SameSite, time::Duration},
};
const NAME: &str = "haven_feed_return";

fn safe_target(raw: &str) -> Option<&str> {
    (raw.is_ascii()
        && raw.len() <= 1800
        && (raw == "/offers" || raw.starts_with("/offers?"))
        && !raw.contains('\\')
        && !raw.chars().any(char::is_control))
    .then_some(raw)
}

pub(super) fn remember(cx: &Cx, target: &str) {
    let Some(target) = safe_target(target) else {
        return;
    };
    let mut cookie = Cookie::new(NAME, hex::encode(target));
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(!cfg!(debug_assertions));
    cookie.set_max_age(Duration::seconds(900));
    topcoat::cookie::cookies(cx).add(cookie);
    topcoat::router::response::response_headers(cx).append(
        topcoat::router::header::HeaderName::from_static("cache-control"),
        topcoat::router::header::HeaderValue::from_static("private, no-store"),
    );
}

pub(super) fn destination(cx: &Cx) -> String {
    let jar = topcoat::cookie::cookies(cx);
    let target = jar.get(NAME).and_then(|cookie| {
        let raw = cookie.value();
        if raw.len() > 3600 {
            return None;
        }
        let decoded = String::from_utf8(hex::decode(raw).ok()?).ok()?;
        safe_target(&decoded).map(str::to_string)
    });
    let mut removal = Cookie::new(NAME, "");
    removal.set_path("/");
    jar.remove(removal);
    target.unwrap_or_else(|| "/offers".to_string())
}

#[cfg(test)]
mod tests {
    use super::safe_target;
    #[test]
    fn only_feed_paths_and_queries_are_valid_return_destinations() {
        for valid in [
            "/offers",
            "/offers?scope=saved&category=food-groceries",
            "/offers?scope=inquired&q=rice",
        ] {
            assert_eq!(safe_target(valid), Some(valid));
        }
        for invalid in [
            "https://example.com",
            "//example.com",
            "/offers/elsewhere",
            "/offers?x=bad\\path",
            "/offers?x=bad\r\nheader",
        ] {
            assert_eq!(safe_target(invalid), None);
        }
    }
}
