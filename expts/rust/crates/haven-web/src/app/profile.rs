//! Profile shell layout and overview at `/profile`.

pub mod account;
pub mod edit;
pub mod offers;

use crate::app::components::button::{ButtonVariant, button_link};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

/// Query parameters on `/profile`.
#[topcoat::router::query_params]
pub struct ProfileQuery {
    /// Indicates whether a profile update just succeeded.
    pub updated: Option<String>,
}

#[topcoat::router::layout]
pub async fn profile_layout(cx: &Cx, slot: topcoat::router::Slot<'_>) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let path = topcoat::router::request::uri(cx).path();
    let is_overview =
        path == "/profile" || path == "/profile/" || path.starts_with("/profile/edit");
    let is_offers = path.starts_with("/profile/offers");
    let is_account = path.starts_with("/profile/account");

    let public_profile_href = format!("/profiles/{}", user.handle.as_str());
    let display_name_str = user.display_name.as_str().to_string();
    let handle_str = format!("@{}", user.handle.as_str());
    let bio_str = user.bio.as_ref().map(|b| b.as_str().to_string());

    Ok(view! {
        <div class="profile-shell">
            <meta name="robots" content="noindex" />
            <div class="profile-shell-header">
                <div class="profile-identity">
                    <h1 class="profile-display-name">(display_name_str)</h1>
                    <span class="profile-handle text-muted">(handle_str)</span>
                </div>
                if let Some(bio) = bio_str {
                    <p class="profile-bio">(bio)</p>
                } else {
                    <p class="profile-bio text-muted">"No bio added yet."</p>
                }
                <div class="profile-header-actions">
                    <a href=(public_profile_href) class="text-muted public-profile-link">
                        "View public profile →"
                    </a>
                </div>
            </div>

            <nav class="profile-nav" aria-label="Profile Sections">
                <a
                    href="/profile"
                    class=(if is_overview { "profile-tab active" } else { "profile-tab" })
                    aria-current=(if is_overview { Some("page") } else { None })
                >
                    "Overview"
                </a>
                <a
                    href="/profile/offers"
                    class=(if is_offers { "profile-tab active" } else { "profile-tab" })
                    aria-current=(if is_offers { Some("page") } else { None })
                >
                    "Offers"
                </a>
                <a
                    href="/profile/account"
                    class=(if is_account { "profile-tab active" } else { "profile-tab" })
                    aria-current=(if is_account { Some("page") } else { None })
                >
                    "Account"
                </a>
            </nav>

            <div class="profile-shell-content">
                (slot)
            </div>
        </div>
    })
}

#[topcoat::router::page]
pub async fn profile_overview(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let query = topcoat::router::query_params::<ProfileQuery>(cx).ok();
    let is_updated = query.as_ref().and_then(|q| q.updated.as_deref()) == Some("1");

    let offers = crate::app::state::registry(cx)
        .offers()
        .find_by_user(user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    let offers_count = offers.len();

    Ok(view! {
        <div class="profile-overview-section">
            if is_updated {
                <div class="notice-banner" role="status">
                    <p>"Your profile has been updated."</p>
                </div>
            }

            <div class="overview-grid">
                <div class="overview-card">
                    <div class="overview-card-header">
                        <h2 class="section-title">"Identity"</h2>
                        button_link(
                            href: "/profile/edit",
                            text: "Edit Profile",
                            variant: ButtonVariant::Secondary,
                        )
                    </div>
                    <p class="text-muted">
                        "Your identity as seen by people viewing your offers and public profile."
                    </p>
                </div>

                <div class="overview-card">
                    <div class="overview-card-header">
                        <h2 class="section-title">"Your Offers"</h2>
                        button_link(
                            href: "/profile/offers",
                            text: "View All Offers",
                            variant: ButtonVariant::Secondary,
                        )
                    </div>
                    <p class="text-muted">
                        if offers_count == 0 {
                            "You haven't created any offers yet."
                        } else {
                            (format!("You have {offers_count} active offer(s)."))
                        }
                    </p>
                    <div class="overview-card-actions">
                        button_link(
                            href: "/profile/offers/new",
                            text: "Create an Offer",
                            variant: ButtonVariant::Primary,
                        )
                    </div>
                </div>
            </div>
        </div>
    })
}
