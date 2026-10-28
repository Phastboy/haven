use super::*;
use topcoat::{context::Cx, view::ViewExt};

#[tokio::test]
async fn selected_and_unselected_chips_are_links_with_current_state_only_when_selected() {
    for selected in [false, true] {
        let cx = Cx::default();
        let __cx = &cx;
        let rendered = view! { chip(props: ChipData {
            label: "Nearby".to_string(), href: "/browse?near=1".to_string(), selected,
            accessible_label: None,
        }) };
        let html = rendered.single().await.expect("render chip").render(__cx);
        assert!(html.contains("<a "));
        assert!(html.contains("href=\"/browse?near=1\""));
        assert!(html.contains("Nearby"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("role=\"button\""));
        assert_eq!(html.contains("aria-current=\"true\""), selected);
        assert!(!html.contains("aria-label="));
        for forbidden in ["<script", "<style", " style=", " onclick="] {
            assert!(!html.contains(forbidden));
        }
    }
}

#[tokio::test]
async fn accessible_label_and_visible_text_are_escaped() {
    let cx = Cx::default();
    let __cx = &cx;
    let rendered = view! { chip(props: ChipData {
        label: "<Nearby> & more".to_string(), href: "/browse?near=1&sort=recent".to_string(),
        selected: false, accessible_label: Some("Browse \"nearby\" places".to_string()),
    }) };
    let html = rendered.single().await.expect("render chip").render(__cx);
    assert!(html.contains("&lt;Nearby&gt; &amp; more"));
    assert!(html.contains("href=\"/browse?near=1&amp;sort=recent\""));
    assert!(html.contains("aria-label=\"Browse &quot;nearby&quot; places\""));
}
