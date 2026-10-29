use http::Request;
use topcoat::{
    context::Cx,
    router::{Body, to_bytes},
    view::{ViewExt, view},
};

use crate::app::components::nav::{
    RouteKind, ViewerKind,
    app_shell::{AppShellData, app_shell},
};

#[tokio::test]
#[allow(clippy::expect_used, reason = "layout render test")]
async fn app_shell_renders_signed_in_visitor_and_auth_navigation() {
    let cx = Cx::default();
    let __cx = &cx;

    let signed_in = view! {
        app_shell(
            props: AppShellData {
                viewer: ViewerKind::SignedIn,
                route: "/conversations",
                kind: RouteKind::Main,
                back_href: String::new(),
                auth_page: false,
            },
            <p>"Content"</p>
        )
    };
    let signed_in_html = signed_in.single().await.expect("shell").render(__cx);
    assert!(signed_in_html.contains("aria-label=\"Messages\""));
    assert!(signed_in_html.contains("aria-current=\"page\""));
    assert!(signed_in_html.contains("aria-label=\"Unread messages\""));
    assert!(signed_in_html.contains("aria-label=\"Profile\""));
    assert_eq!(signed_in_html.matches("data-unread-indicator").count(), 2);
    assert_eq!(signed_in_html.matches("hidden=\"hidden\"").count(), 2);
    assert!(signed_in_html.contains("app-bottom-tabs"));
    assert!(signed_in_html.contains("app-rail"));
    assert!(!signed_in_html.contains("site-header"));

    let visitor = view! {
        app_shell(
            props: AppShellData {
                viewer: ViewerKind::Visitor,
                route: "/",
                kind: RouteKind::Main,
                back_href: String::new(),
                auth_page: false,
            },
            <p>"Content"</p>
        )
    };
    let visitor_html = visitor.single().await.expect("shell").render(__cx);
    assert!(visitor_html.contains("href=\"/offers\" aria-current=\"page\""));
    assert!(visitor_html.contains("href=\"/auth/sign-in\""));
    assert!(visitor_html.contains("app-bottom-tabs"));
    assert!(visitor_html.contains("app-rail"));
    assert!(!visitor_html.contains("aria-label=\"Messages\""));
    assert!(!visitor_html.contains("aria-label=\"Profile\""));
    assert!(!visitor_html.contains("site-header"));

    let auth_page = view! {
        app_shell(
            props: AppShellData {
                viewer: ViewerKind::Visitor,
                route: "/auth/sign-in",
                kind: RouteKind::Child,
                back_href: "/".into(),
                auth_page: true,
            },
            <p>"Content"</p>
        )
    };
    let auth_html = auth_page.single().await.expect("shell").render(__cx);
    assert!(auth_html.contains("href=\"/offers\""));
    assert!(!auth_html.contains("href=\"/auth/sign-in\""));
    assert!(!auth_html.contains("aria-label=\"Messages\""));
    assert!(!auth_html.contains("aria-label=\"Profile\""));
    assert!(auth_html.contains("app-mobile-back-bar"));
    assert!(auth_html.contains("app-shell-back-link"));
    assert!(auth_html.contains("app-rail"));
    assert!(!auth_html.contains("app-bottom-tabs"));
    assert!(!auth_html.contains("site-header"));
}

#[tokio::test]
#[allow(clippy::expect_used, reason = "layout integration test")]
async fn root_layout_wraps_multiple_routes_in_the_same_rail() {
    let (router, _) = crate::app::offers::tests::setup_test_app();

    for path in ["/auth/sign-in", "/auth/sent"] {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .expect("request");
        let response = router.handle(request).await;
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body bytes");
        let html = String::from_utf8(bytes.to_vec()).expect("UTF-8 HTML");
        assert!(
            html.contains("class=\"app-rail\""),
            "missing rail on {path}"
        );
        assert!(
            html.contains("class=\"app-shell"),
            "missing shell on {path}: {html}"
        );
        assert!(!html.contains("site-header"), "old top nav on {path}");
    }
}
