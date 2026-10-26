use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

use super::ViewerKind;

pub struct BottomTabsData<'a> {
    pub viewer: ViewerKind,
    pub route: &'a str,
    pub has_unread: bool,
}

#[component]
pub async fn bottom_tabs(props: BottomTabsData<'_>) -> TopcoatResult<impl View> {
    let offers_active =
        props.route == "/" || props.route == "/offers" || props.route.starts_with("/offers/");
    let profile_active = props.route == "/profile" || props.route.starts_with("/profile/");
    let messages_active =
        props.route == "/conversations" || props.route.starts_with("/conversations/");
    Ok(view! {
        <nav class="app-bottom-tabs" aria-label="Mobile navigation">
            <a href="/offers" class=(if offers_active { "app-bottom-tab active" } else { "app-bottom-tab" }) aria-current=(if offers_active { Some("page") } else { None })>
                <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg><span>"Offers"</span>
            </a>
            if props.viewer == ViewerKind::SignedIn {
                <a href="/conversations" class=(if messages_active { "app-bottom-tab active" } else { "app-bottom-tab" }) aria-current=(if messages_active { Some("page") } else { None })>
                    <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg><span>"Messages" if props.has_unread { <i aria-label="Unread messages"></i> }</span>
                </a>
                <a href="/profile" class=(if profile_active { "app-bottom-tab active" } else { "app-bottom-tab" }) aria-current=(if profile_active { Some("page") } else { None })>
                    <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></svg><span>"Profile"</span>
                </a>
            } else {
                <a href="/auth/sign-in" class="app-bottom-tab"><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></svg><span>"Sign in"</span></a>
            }
        </nav>
    })
}
