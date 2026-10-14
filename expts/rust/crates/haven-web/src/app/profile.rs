//! Profile shell layout and overview at `/profile`.

pub mod account;
pub mod avatar;
pub mod edit;
pub mod offers;

use crate::app::components::{
    identity_block::{IdentityBlockData, identity_block},
    profile_row::{ProfileRowData, RowIcon},
    row_group::{ProfileGroupData, row_group},
};
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
    let _user = crate::app::auth::guard::require_owner_auth(cx).await?;

    Ok(view! {
        <div class="profile-page-shell">
            <meta name="robots" content="noindex" />
            (slot)
        </div>
    })
}

#[topcoat::router::page]
pub async fn profile_overview(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let query = topcoat::router::query_params::<ProfileQuery>(cx).ok();
    let is_updated = query.as_ref().and_then(|q| q.updated.as_deref()) == Some("1");

    let joined_date = user.created_at.format("Joined %B %Y").to_string();
    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let (avatar_url, avatar_srcset) = match user.avatar_public_id.as_ref() {
        Some(public_id) => (
            Some(crate::app::components::image::avatar_url(
                cloud_name,
                public_id.as_str(),
            )),
            Some(crate::app::components::image::avatar_srcset(
                cloud_name,
                public_id.as_str(),
            )),
        ),
        None => (None, None),
    };

    let identity_data = IdentityBlockData {
        display_name: user.display_name.as_str().to_string(),
        handle: user.handle.as_str().to_string(),
        joined_date,
        avatar_url,
        avatar_srcset,
        bio: user.bio.as_ref().map(|b| b.as_str().to_string()),
    };

    let profile_group_data = ProfileGroupData {
        label: "PROFILE",
        rows: vec![
            ProfileRowData {
                icon: RowIcon::PublicProfile,
                title: "Public Profile".to_string(),
                description: "View how others see your profile".to_string(),
                href: format!("/profiles/{}", user.handle.as_str()),
                is_post: false,
            },
            ProfileRowData {
                icon: RowIcon::EditProfile,
                title: "Edit Profile".to_string(),
                description: "Update display name, handle, or bio".to_string(),
                href: "/profile/edit".to_string(),
                is_post: false,
            },
        ],
    };

    let offers_group_data = ProfileGroupData {
        label: "OFFERS",
        rows: vec![
            ProfileRowData {
                icon: RowIcon::Offers,
                title: "Your Offers".to_string(),
                description: "Manage your active listings".to_string(),
                href: "/profile/offers".to_string(),
                is_post: false,
            },
            ProfileRowData {
                icon: RowIcon::CreateOffer,
                title: "Create an Offer".to_string(),
                description: "Publish a new offer for others to see".to_string(),
                href: "/profile/offers/new".to_string(),
                is_post: false,
            },
        ],
    };

    let account_group_data = ProfileGroupData {
        label: "ACCOUNT",
        rows: vec![
            ProfileRowData {
                icon: RowIcon::Account,
                title: "Account & Session".to_string(),
                description: "Manage signed-in email and session".to_string(),
                href: "/profile/account".to_string(),
                is_post: false,
            },
            ProfileRowData {
                icon: RowIcon::SignOut,
                title: "Sign Out".to_string(),
                description: "End your current session".to_string(),
                href: "/auth/sign-out".to_string(),
                is_post: true,
            },
        ],
    };

    Ok(view! {
        <div class="profile-overview">
            if is_updated {
                <div class="notice-banner" role="status">
                    <p>"Your profile has been updated."</p>
                </div>
            }

            identity_block(data: identity_data)

            <div class="profile-sections">
                row_group(group: profile_group_data)
                row_group(group: offers_group_data)
                row_group(group: account_group_data)
            </div>
        </div>
    })
}
