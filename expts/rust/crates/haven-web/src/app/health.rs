use topcoat::{Result as TopcoatResult, router::route};

#[route(GET)]
pub async fn health() -> TopcoatResult<&'static str> {
    Ok("ok")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test suite assertions")]

    #[tokio::test]
    async fn test_health_route() {
        let router = crate::app::router().build();
        let request = http::Request::builder()
            .method("GET")
            .uri("/health")
            .body(topcoat::router::Body::empty())
            .unwrap();
        let response = router.handle(request).await;
        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
