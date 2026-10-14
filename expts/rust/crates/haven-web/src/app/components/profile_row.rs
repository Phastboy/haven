//! Reusable slot-based row component for profile settings and navigation.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Icon choices for row leading slot, adhering to Haven's monochrome-first icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowIcon {
    PublicProfile,
    EditProfile,
    Offers,
    CreateOffer,
    Account,
    SignOut,
}

/// Renders the semantic monochrome SVG icon for the row's leading slot.
#[component]
pub async fn row_icon(icon: RowIcon) -> TopcoatResult<impl View> {
    Ok(view! {
        match icon {
            RowIcon::PublicProfile => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                </svg>
            }
            RowIcon::EditProfile => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                </svg>
            }
            RowIcon::Offers => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
                    <line x1="7" y1="7" x2="7.01" y2="7" />
                </svg>
            }
            RowIcon::CreateOffer => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
            }
            RowIcon::Account => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                    <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                </svg>
            }
            RowIcon::SignOut => {
                <svg class="row-icon-svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                    <polyline points="16 17 21 12 16 7" />
                    <line x1="21" y1="12" x2="9" y2="12" />
                </svg>
            }
        }
    })
}

/// Typed data defining a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRowData {
    /// Semantic icon shown in leading slot.
    pub icon: RowIcon,
    /// Primary title text.
    pub title: String,
    /// Optional secondary muted description text.
    pub description: Option<String>,
    /// Target destination URL (or form action).
    pub href: String,
    /// If true, renders a POST form submit button instead of a GET link.
    pub is_post: bool,
    /// If set, renders a button triggering a native Popover dialog instead of navigating.
    pub popovertarget: Option<String>,
}

/// Renders a single row conforming to Haven's 44px min-height touch target and slot arrangement.
#[component]
pub async fn profile_row(data: ProfileRowData) -> TopcoatResult<impl View> {
    Ok(view! {
        if let Some(target) = data.popovertarget {
            <button type="button" class="profile-row profile-row-btn" popovertarget=(target)>
                <div class="row-leading-slot" aria-hidden="true">
                    row_icon(icon: data.icon)
                </div>
                <div class="row-text-slot">
                    <span class="row-title">(data.title)</span>
                    if let Some(desc) = data.description {
                        <span class="row-description text-muted">(desc)</span>
                    }
                </div>
            </button>
        } else if data.is_post {
            <form action=(data.href) method="post" class="profile-row-form">
                <button type="submit" class="profile-row profile-row-btn">
                    <div class="row-leading-slot" aria-hidden="true">
                        row_icon(icon: data.icon)
                    </div>
                    <div class="row-text-slot">
                        <span class="row-title">(data.title)</span>
                        if let Some(desc) = data.description {
                            <span class="row-description text-muted">(desc)</span>
                        }
                    </div>
                    <div class="row-trailing-slot text-muted" aria-hidden="true">
                        <svg class="row-chevron-svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <polyline points="9 18 15 12 9 6" />
                        </svg>
                    </div>
                </button>
            </form>
        } else {
            <a href=(data.href) class="profile-row">
                <div class="row-leading-slot" aria-hidden="true">
                    row_icon(icon: data.icon)
                </div>
                <div class="row-text-slot">
                    <span class="row-title">(data.title)</span>
                    if let Some(desc) = data.description {
                        <span class="row-description text-muted">(desc)</span>
                    }
                </div>
                <div class="row-trailing-slot text-muted" aria-hidden="true">
                    <svg class="row-chevron-svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="9 18 15 12 9 6" />
                    </svg>
                </div>
            </a>
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_profile_row_renders_link() {
        let cx = Cx::default();
        let __cx = &cx;
        let row = view! {
            profile_row(data: ProfileRowData {
                icon: RowIcon::EditProfile,
                title: "Edit Profile".to_string(),
                description: Some("Update display name and bio.".to_string()),
                href: "/profile/edit".to_string(),
                is_post: false,
                popovertarget: None,
            })
        };

        let html = row.single().await.unwrap().render(__cx);
        assert!(html.contains("<a href=\"/profile/edit\" class=\"profile-row\">"));
        assert!(html.contains("Edit Profile"));
        assert!(html.contains("Update display name and bio."));
        assert!(html.contains("row-chevron-svg"));
        assert!(!html.contains("<form"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_profile_row_renders_post_form() {
        let cx = Cx::default();
        let __cx = &cx;
        let row = view! {
            profile_row(data: ProfileRowData {
                icon: RowIcon::SignOut,
                title: "Sign Out".to_string(),
                description: Some("End your current session.".to_string()),
                href: "/auth/sign-out".to_string(),
                is_post: true,
                popovertarget: None,
            })
        };

        let html = row.single().await.unwrap().render(__cx);
        assert!(html.contains(
            "<form action=\"/auth/sign-out\" method=\"post\" class=\"profile-row-form\">"
        ));
        assert!(html.contains("Sign Out"));
        assert!(html.contains("End your current session."));
        assert!(html.contains("row-chevron-svg"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_profile_row_renders_popover_button() {
        let cx = Cx::default();
        let __cx = &cx;
        let row = view! {
            profile_row(data: ProfileRowData {
                icon: RowIcon::SignOut,
                title: "Sign out".to_string(),
                description: None,
                href: "/auth/sign-out".to_string(),
                is_post: false,
                popovertarget: Some("signout-dialog".to_string()),
            })
        };

        let html = row.single().await.unwrap().render(__cx);
        assert!(html.contains("<button type=\"button\" class=\"profile-row profile-row-btn\" popovertarget=\"signout-dialog\">"));
        assert!(html.contains("Sign out"));
        assert!(!html.contains("row-chevron-svg"));
    }
}
