//! Reusable row group container component for profile sections.

use crate::app::components::profile_row::{ProfileRowData, profile_row};
use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Typed group data defining a section of rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileGroupData {
    /// Uppercase group label shown above the container.
    pub label: &'static str,
    /// List of rows contained in this group.
    pub rows: Vec<ProfileRowData>,
}

/// Renders a group of rows with an uppercase letter-spaced label and rounded hairline container.
#[component]
pub async fn row_group(group: ProfileGroupData) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class="row-group">
            <span class="row-group-label text-muted">(group.label)</span>
            <div class="row-group-container">
                for row_data in group.rows {
                    profile_row(data: row_data)
                }
            </div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::components::profile_row::RowIcon;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_row_group_renders_label_and_rows() {
        let cx = Cx::default();
        let __cx = &cx;
        let group = view! {
            row_group(group: ProfileGroupData {
                label: "OFFERS",
                rows: vec![
                    ProfileRowData {
                        icon: RowIcon::Offers,
                        title: "Your Offers".to_string(),
                        description: "Manage your listings.".to_string(),
                        href: "/profile/offers".to_string(),
                        is_post: false,
                    },
                    ProfileRowData {
                        icon: RowIcon::CreateOffer,
                        title: "Create an Offer".to_string(),
                        description: "Publish a new offer.".to_string(),
                        href: "/profile/offers/new".to_string(),
                        is_post: false,
                    },
                ],
            })
        };

        let html = group.single().await.unwrap().render(__cx);
        assert!(html.contains("row-group-label"));
        assert!(html.contains("OFFERS"));
        assert!(html.contains("Your Offers"));
        assert!(html.contains("Create an Offer"));
        assert!(html.contains("href=\"/profile/offers\""));
        assert!(html.contains("href=\"/profile/offers/new\""));
    }
}
