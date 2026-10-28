use topcoat::{
    Result as TopcoatResult,
    router::{content::Css, route},
};

const STYLESHEET: &str = concat!(
    include_str!("../../static/style.css"),
    "\n",
    include_str!("../../static/css/tokens.css"),
    "\n",
    include_str!("../../static/css/components/nav/app_shell.css"),
    "\n",
    include_str!("../../static/css/components/nav/rail.css"),
    "\n",
    include_str!("../../static/css/components/nav/bottom_tabs.css"),
    "\n",
    include_str!("../../static/css/components/nav/mobile_back_bar.css"),
    "\n",
    include_str!("../../static/css/components/nav/navigation_breakpoints.css"),
    "\n",
    include_str!("../../static/css/pages/landing.css"),
    "\n",
    include_str!("../../static/css/pages/landing-responsive.css"),
);

/// Serves the central Haven stylesheet with strict CSP compliance.
#[route(GET "/style.css")]
pub async fn stylesheet() -> TopcoatResult<Css<&'static str>> {
    Ok(Css(STYLESHEET))
}

/// Serves the CSP-compatible offer cover delivery fallback.
#[route(GET "/cover-images.js")]
pub async fn cover_images_script() -> TopcoatResult<topcoat::router::content::Js<&'static str>> {
    Ok(topcoat::router::content::Js(include_str!(
        "../../static/cover-images.js"
    )))
}

#[cfg(test)]
mod tests {
    use http::Request;
    use topcoat::{
        router::{Body, RouterBuilderDiscoverExt},
        runtime::RouterBuilderRuntimeExt,
    };

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_style_route_serves_css() {
        let router = crate::app::router().unwrap().discover().runtime().build();
        let request = Request::builder()
            .method("GET")
            .uri("/style.css")
            .body(Body::empty())
            .unwrap();
        let response = router.handle(request).await;
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "text/css; charset=utf-8"
        );
    }
}
