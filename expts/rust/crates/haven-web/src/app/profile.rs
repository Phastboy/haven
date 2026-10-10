//! Profile dashboard route at `/profile`.

#![allow(clippy::too_many_lines, reason = "Profile dashboard page layout")]

pub mod edit;

use crate::app::components::button::{ButtonVariant, button_link};
use crate::app::components::offer_card::offer_card;
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

#[topcoat::router::page]
pub async fn profile_page(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let query = topcoat::router::query_params::<ProfileQuery>(cx).ok();
    let is_updated = query.as_ref().and_then(|q| q.updated.as_deref()) == Some("1");

    let offers = crate::app::state::registry(cx)
        .offers()
        .find_by_user(user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    let public_profile_href = format!("/profiles/{}", user.handle.as_str());
    let display_name_str = user.display_name.as_str().to_string();
    let handle_str = format!("@{}", user.handle.as_str());
    let bio_str = user.bio.as_ref().map(|b| b.as_str().to_string());
    let offers_count = offers.len();

    Ok(view! {
        <div class="profile-dashboard">
            <meta name="robots" content="noindex" />
            if is_updated {
                <div class="notice-banner" role="status">
                    <p>"Your profile has been updated."</p>
                </div>
            }

            <div class="profile-dashboard-header">
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
                    button_link(
                        href: "/profile/edit",
                        text: "Edit Profile",
                        variant: ButtonVariant::Secondary,
                    )
                    <a href=(public_profile_href) class="text-muted public-profile-link">
                        "View public profile →"
                    </a>
                </div>
            </div>

            <section class="profile-section">
                <div class="page-header-row">
                    <div>
                        <h2 class="section-title">"Your Offers"</h2>
                        <p class="text-muted">
                            if offers_count == 0 {
                                "You haven't created any offers yet."
                            } else {
                                (format!("You have {offers_count} active offer(s)."))
                            }
                        </p>
                    </div>
                    button_link(
                        href: "/offers/manage/new",
                        text: "Create Offer",
                        variant: ButtonVariant::Primary,
                    )
                </div>

                if offers_count > 0 {
                    <div class="profile-offers-preview">
                        <ul class="offer-list" role="list">
                            for offer in offers.iter().take(5) {
                                offer_card(
                                    title: offer.title.clone(),
                                    href: format!("/offers/{}", offer.slug),
                                    price_text: if offer.price.as_i32() == 0 {
                                        "Free".to_string()
                                    } else {
                                        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
                                    },
                                    description: offer.description.clone(),
                                    edit_href: Some(format!("/offers/manage/{}/edit", offer.id)),
                                    delete_action: Some(format!("/offers/manage/{}/delete", offer.id)),
                                )
                            }
                        </ul>
                        if offers_count > 5 {
                            <div class="profile-manage-link-bar">
                                <a href="/offers/manage" class="btn btn-secondary">
                                    "Manage All Offers (" (offers_count.to_string()) ")"
                                </a>
                            </div>
                        }
                    </div>
                }
            </section>

            <section class="profile-section profile-account-section">
                <h2 class="section-title">"Account & Session"</h2>
                <p class="text-muted">
                    "Signed in to Haven."
                </p>
                <form action="/auth/sign-out" method="post" class="sign-out-form">
                    <button type="submit" class="btn btn-secondary">
                        "Sign Out"
                    </button>
                </form>
            </section>
        </div>
    })
}
