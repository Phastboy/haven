//! Private unread status, loaded independently of page rendering.

use topcoat::{Result, context::Cx, router::content::Json};

/// The signed-in person's current navigation indicator state.
#[derive(serde::Serialize)]
pub struct UnreadStatus {
    /// Identity bound to this private response, used to reject stale account state.
    pub person_id: String,
    /// True when at least one conversation has an unread latest incoming message.
    pub has_unread: bool,
}

/// Returns unread state for the authenticated session without rendering a page.
///
/// # Errors
/// Returns 401 without a valid session, or a server error if storage is unavailable.
#[topcoat::router::route(GET "/unread-status")]
pub async fn unread_status(cx: &Cx) -> Result<Json<UnreadStatus>> {
    let headers = topcoat::router::response::response_headers(cx);
    headers.append(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("private, no-store"),
    );
    headers.append(http::header::VARY, http::HeaderValue::from_static("Cookie"));
    headers.append(
        http::HeaderName::from_static("x-robots-tag"),
        http::HeaderValue::from_static("noindex"),
    );
    let person = crate::app::auth::guard::current_user(cx)
        .await?
        .ok_or_else(topcoat::router::error::unauthorized)?;
    let has_unread = crate::app::state::registry(cx)
        .conversations()
        .unread_exists(person.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    Ok(Json(UnreadStatus {
        person_id: person.id.to_string(),
        has_unread,
    }))
}

/// Serves the initial unread-status enhancement under the site's script CSP.
#[topcoat::router::route(GET "/unread-status.js")]
pub async fn unread_script() -> Result<topcoat::router::content::Js<&'static str>> {
    Ok(topcoat::router::content::Js(include_str!(
        "../../static/unread-status.js"
    )))
}
