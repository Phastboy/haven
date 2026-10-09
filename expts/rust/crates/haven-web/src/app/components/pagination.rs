use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// Keyset pagination navigation controls for forward and backward traversal.
#[component]
pub async fn pagination_nav(
    #[default] prev_href: Option<String>,
    #[default] next_href: Option<String>,
) -> TopcoatResult<impl View> {
    Ok(view! {
        <nav class="pagination-nav" aria-label="Pagination">
            if let Some(href) = prev_href {
                <a href=(href) class="btn btn-secondary">"← Previous"</a>
            } else {
                <span></span>
            }
            if let Some(href) = next_href {
                <a href=(href) class="btn btn-secondary">"Next →"</a>
            } else {
                <span></span>
            }
        </nav>
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
    async fn pagination_nav_renders_navigation_links() {
        let cx = Cx::default();
        let __cx = &cx;
        let nav = view! {
            pagination_nav(
                prev_href: Some("/offers?before=cursor1".to_string()),
                next_href: Some("/offers?after=cursor2".to_string()),
            )
        };

        let html = nav.single().await.unwrap().render(__cx);
        assert!(html.contains("href=\"/offers?before=cursor1\""));
        assert!(html.contains("← Previous"));
        assert!(html.contains("href=\"/offers?after=cursor2\""));
        assert!(html.contains("Next →"));
    }
}
