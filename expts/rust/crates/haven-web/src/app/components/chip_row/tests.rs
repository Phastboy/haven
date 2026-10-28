use super::*;
use topcoat::{context::Cx, view::ViewExt};

#[tokio::test]
async fn named_navigation_contains_an_ordered_list_of_anchor_chips() {
    let cx = Cx::default();
    let __cx = &cx;
    let chips = ["Nearby", "Everywhere"]
        .into_iter()
        .enumerate()
        .map(|(i, label)| ChipData {
            label: label.to_string(),
            href: format!("/browse?area={i}"),
            selected: i == 0,
            accessible_label: None,
        })
        .collect();
    let rendered = view! { chip_row(props: ChipRowData { chips, accessible_name: "Choose an area".to_string() }) };
    let html = rendered.single().await.expect("render row").render(__cx);
    assert!(html.contains("<nav class=\"chip-row\" aria-label=\"Choose an area\""));
    assert!(html.contains("<ul class=\"chip-row-list\" role=\"list\""));
    assert_eq!(html.matches("<li>").count(), 2);
    assert_eq!(html.matches("<a ").count(), 2);
    assert_eq!(html.matches("aria-current=").count(), 1);
    assert!(
        html.find("Nearby").expect("first label") < html.find("Everywhere").expect("second label")
    );
    for forbidden in ["<button", "<script", "<style", " style=", " onclick="] {
        assert!(!html.contains(forbidden));
    }
}

#[tokio::test]
async fn an_empty_slot_does_not_add_an_empty_navigation_landmark() {
    let cx = Cx::default();
    let __cx = &cx;
    let rendered = view! { chip_row(props: ChipRowData { chips: Vec::new(), accessible_name: "Choose an area".to_string() }) };
    let html = rendered
        .single()
        .await
        .expect("render empty row")
        .render(__cx);
    assert!(!html.contains("<nav"));
    assert!(!html.contains("<ul"));
}
