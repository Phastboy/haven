//! Save or unsave a public offer.

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::Form, error::see_other},
};

/// Toggles a saved offer from the browser runtime.
#[topcoat::runtime::procedure]
pub async fn toggle_save_in_place(cx: &Cx, slug_raw: String, saved: bool) -> TopcoatResult<bool> {
    toggle(cx, &slug_raw, saved).await?;
    Ok(!saved)
}

#[derive(serde::Deserialize)]
pub struct SaveForm {
    pub action: String,
    pub return_to: Option<String>,
}

/// Keeps save redirects on the public offers feed.
#[must_use]
pub fn safe_return_to(candidate: Option<&str>, slug: &str) -> String {
    let fallback = format!("/offers/{slug}");
    let Some(path) = candidate else {
        return fallback;
    };
    let valid = path.starts_with("/offers")
        && !path.starts_with("//")
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path
            .strip_prefix("/offers")
            .is_some_and(|tail| tail.is_empty() || tail.starts_with('/') || tail.starts_with('?'));
    if valid { path.to_string() } else { fallback }
}

#[topcoat::router::page(POST)]
pub async fn toggle_save(cx: &Cx, form: Form<SaveForm>) -> TopcoatResult<()> {
    let slug_raw: &str = topcoat::router::path_param::<super::Slug>(cx);
    let saved = match form.0.action.as_str() {
        "save" => true,
        "unsave" => false,
        _ => return Err(topcoat::router::error::bad_request("invalid save action").into()),
    };
    toggle(cx, slug_raw, saved).await?;
    let return_to = format!(
        "{}#offer-{}",
        safe_return_to(form.0.return_to.as_deref(), slug_raw),
        slug_raw
    );
    Err(see_other(return_to).into())
}

async fn toggle(cx: &Cx, slug_raw: &str, saved: bool) -> TopcoatResult<()> {
    let slug = haven_domain::offer::OfferSlug::parse(slug_raw)
        .map_err(|_| topcoat::router::error::not_found())?;
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let registry = crate::app::state::registry(cx);
    let offer = registry
        .offers()
        .find_by_slug(&slug)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .filter(haven_domain::offer::Offer::is_public)
        .ok_or_else(topcoat::router::error::not_found)?;
    if saved {
        registry.saved_offers().save(user.id, offer.id).await
    } else {
        registry.saved_offers().unsave(user.id, offer.id).await
    }
    .map_err(crate::app::state::map_repo_err)?;
    Ok(())
}

#[cfg(test)]
mod return_to_tests {
    use super::safe_return_to;

    #[test]
    fn accepts_only_relative_offers_paths() {
        assert_eq!(
            safe_return_to(Some("/offers?q=test"), "chair"),
            "/offers?q=test"
        );
        for unsafe_path in [
            "//evil.com",
            "/profile",
            "javascript:alert(1)",
            "https://evil.com",
            "/offers\\evil",
            "/offers\r\nLocation: /evil",
        ] {
            assert_eq!(safe_return_to(Some(unsafe_path), "chair"), "/offers/chair");
        }
    }
}
