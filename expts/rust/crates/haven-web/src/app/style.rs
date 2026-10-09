use topcoat::{
    Result as TopcoatResult,
    router::{content::Css, route},
};

const STYLESHEET: &str = include_str!("../../static/style.css");

/// Serves the central Haven stylesheet with strict CSP compliance.
#[route(GET "/style.css")]
pub async fn stylesheet() -> TopcoatResult<Css<&'static str>> {
    Ok(Css(STYLESHEET))
}

#[cfg(test)]
mod tests {
    use http::Request;
    use topcoat::router::{Body, RouterBuilderDiscoverExt};

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertions")]
    async fn test_style_route_serves_css() {
        let router = crate::app::router().discover().build();
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
