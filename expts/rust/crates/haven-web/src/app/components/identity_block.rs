//! Shared identity block component for profile views.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Typed props for [`identity_block`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityBlockData {
    /// Person's public display name.
    pub display_name: String,
    /// Public username handle (e.g. "alex-wood" or "@alex-wood").
    pub handle: String,
    /// Formatted join date (e.g. "Joined October 2026").
    pub joined_date: String,
    /// Optional avatar image URL (None for neutral initial placeholder).
    pub avatar_url: Option<String>,
    /// Optional avatar srcset attribute for high-DPR displays (e.g. "1x, 2x").
    pub avatar_srcset: Option<String>,
    /// Optional biography paragraph.
    pub bio: Option<String>,
}

/// Computes the single-character initial placeholder for the avatar slot.
#[must_use]
pub fn extract_avatar_initial(display_name: &str) -> String {
    display_name
        .trim()
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string())
}

/// Renders a centered identity block with avatar slot, display name, handle, join date, and optional bio.
#[component]
pub async fn identity_block(data: IdentityBlockData) -> TopcoatResult<impl View> {
    let initial = extract_avatar_initial(&data.display_name);
    let handle_formatted = if data.handle.starts_with('@') {
        data.handle
    } else {
        format!("@{}", data.handle)
    };

    Ok(view! {
        <div class="identity-block">
            <div class="avatar-slot" aria-hidden="true">
                if let Some(avatar_url) = data.avatar_url {
                    if let Some(avatar_srcset) = data.avatar_srcset {
                        <img src=(avatar_url) srcset=(avatar_srcset) alt="" class="avatar-img" />
                    } else {
                        <img src=(avatar_url) alt="" class="avatar-img" />
                    }
                } else {
                    <span class="avatar-initial">(initial)</span>
                }
            </div>

            <h1 class="identity-name">(data.display_name)</h1>
            <span class="identity-handle text-muted">(handle_formatted)</span>

            if let Some(bio) = data.bio {
                <p class="identity-bio">(bio)</p>
            }

            <div class="identity-joined">
                <span class="identity-joined-pill text-muted">(data.joined_date)</span>
            </div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[test]
    fn test_extract_avatar_initial() {
        assert_eq!(extract_avatar_initial("Alex Wood"), "A");
        assert_eq!(extract_avatar_initial("alex"), "A");
        assert_eq!(extract_avatar_initial("  craftsman"), "C");
        assert_eq!(extract_avatar_initial("Élodie"), "É");
        assert_eq!(extract_avatar_initial(""), "?");
        assert_eq!(extract_avatar_initial("   "), "?");
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_identity_block_renders_no_avatar_placeholder() {
        let cx = Cx::default();
        let __cx = &cx;
        let block = view! {
            identity_block(data: IdentityBlockData {
                display_name: "Alex Wood".to_string(),
                handle: "alex-wood".to_string(),
                joined_date: "Joined October 2026".to_string(),
                avatar_url: None,
                avatar_srcset: None,
                bio: Some("Cabinetmaker and restorer.".to_string()),
            })
        };

        let html = block.single().await.unwrap().render(__cx);
        assert!(html.contains("Alex Wood"));
        assert!(html.contains("@alex-wood"));
        assert!(html.contains("Joined October 2026"));
        assert!(html.contains("Cabinetmaker and restorer."));
        assert!(html.contains("avatar-initial"));
        assert!(html.contains(">A<"));
        assert!(!html.contains("<img"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_identity_block_renders_with_avatar_image() {
        let cx = Cx::default();
        let __cx = &cx;
        let block = view! {
            identity_block(data: IdentityBlockData {
                display_name: "Taylor Craft".to_string(),
                handle: "@taylor".to_string(),
                joined_date: "Joined September 2026".to_string(),
                avatar_url: Some("/static/avatar.png".to_string()),
                avatar_srcset: None,
                bio: None,
            })
        };

        let html = block.single().await.unwrap().render(__cx);
        assert!(html.contains("Taylor Craft"));
        assert!(html.contains("@taylor"));
        assert!(html.contains("<img src=\"/static/avatar.png\""));
        assert!(!html.contains("avatar-initial"));
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_identity_block_renders_with_avatar_srcset() {
        let cx = Cx::default();
        let __cx = &cx;
        let block = view! {
            identity_block(data: IdentityBlockData {
                display_name: "Taylor Craft".to_string(),
                handle: "@taylor".to_string(),
                joined_date: "Joined September 2026".to_string(),
                avatar_url: Some("https://res.cloudinary.com/haven/image/private/t_avatar/users/123/avatar".to_string()),
                avatar_srcset: Some("https://res.cloudinary.com/haven/image/private/t_avatar/users/123/avatar 1x, https://res.cloudinary.com/haven/image/private/t_avatar_2x/users/123/avatar 2x".to_string()),
                bio: None,
            })
        };

        let html = block.single().await.unwrap().render(__cx);
        assert!(html.contains("srcset=\"https://res.cloudinary.com/haven/image/private/t_avatar/users/123/avatar 1x, https://res.cloudinary.com/haven/image/private/t_avatar_2x/users/123/avatar 2x\""));
    }
}
