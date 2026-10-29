use topcoat::{
    Result as TopcoatResult,
    view::{Child, View, component, view},
};

use super::{
    RouteKind, ViewerKind,
    bottom_tabs::{BottomTabsData, bottom_tabs},
    mobile_back_bar::{MobileBackBarData, mobile_back_bar},
    rail::{RailData, rail},
};

pub struct AppShellData<'a> {
    pub viewer: ViewerKind,
    pub route: &'a str,
    pub kind: RouteKind,
    pub back_href: String,
    pub auth_page: bool,
}

/// Wraps page content in the shared navigation shell.
///
/// Renders route-kind affordances independently of viewport width. The CSS
/// navigation breakpoint contract selects rail, tabs, and back-bar visibility.
#[component]
pub async fn app_shell(props: AppShellData<'_>, child: Child<'_>) -> TopcoatResult<impl View> {
    Ok(view! {
        <div class=(if props.kind == RouteKind::Child { "app-shell app-shell-child" } else { "app-shell app-shell-main-page" })>
            rail(props: RailData { viewer: props.viewer, route: props.route, auth_page: props.auth_page })
            <div class="app-shell-main">
                if props.kind == RouteKind::Child {
                    <div class="app-shell-back-desktop">
                        <a class="app-shell-back-link" href=(props.back_href.clone())><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5"/><path d="m12 19-7-7 7-7"/></svg><span>"Back"</span></a>
                    </div>
                    mobile_back_bar(props: MobileBackBarData { href: props.back_href })
                }
                <main class="app-shell-content">(child)</main>
                if props.kind == RouteKind::Main {
                    bottom_tabs(props: BottomTabsData { viewer: props.viewer, route: props.route })
                }
            </div>
        </div>
    })
}
