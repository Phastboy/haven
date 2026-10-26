use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

use super::ViewerKind;

pub struct RailData<'a> {
    pub viewer: ViewerKind,
    pub route: &'a str,
    pub auth_page: bool,
    pub has_unread: bool,
}

#[component]
pub async fn rail(props: RailData<'_>) -> TopcoatResult<impl View> {
    let offers_active = !props.auth_page
        && (props.route == "/" || props.route == "/offers" || props.route.starts_with("/offers/"));
    let profile_active = props.route == "/profile" || props.route.starts_with("/profile/");
    let messages_active =
        props.route == "/conversations" || props.route.starts_with("/conversations/");
    let signed_in = props.viewer == ViewerKind::SignedIn;
    Ok(view! {
        <aside class="app-rail" aria-label="Primary navigation">
            <div class="app-rail-top">
                <a class="app-rail-wordmark" href=(if signed_in { "/offers" } else { "/" })><span class="app-rail-wordmark-full">"Haven"</span><span class="app-rail-wordmark-mark">"H"</span></a>
                <nav class="app-rail-links">
                    <a class=(if offers_active { "app-rail-link active" } else { "app-rail-link" }) href="/offers" aria-current=(if offers_active { Some("page") } else { None }) aria-label="Offers">
                        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
                        <span class="app-rail-label">"Offers"</span>
                    </a>
                    if signed_in && !props.auth_page {
                        <a class=(if messages_active { "app-rail-link active" } else { "app-rail-link" }) href="/conversations" aria-current=(if messages_active { Some("page") } else { None }) aria-label="Messages">
                            <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                            <span class="app-rail-label">"Messages"</span>
                            if props.has_unread { <span class="app-rail-unread" aria-label="Unread messages"></span> }
                        </a>
                        <a class=(if profile_active { "app-rail-link active" } else { "app-rail-link" }) href="/profile" aria-current=(if profile_active { Some("page") } else { None }) aria-label="Profile">
                            <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></svg>
                            <span class="app-rail-label">"Profile"</span>
                        </a>
                    }
                </nav>
            </div>
            if !signed_in && !props.auth_page {
                <div class="app-rail-auth">
                    <a class="app-rail-signin" href="/auth/sign-in" aria-label="Sign in">
                        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></svg>
                        <span class="app-rail-label">"Sign in"</span>
                    </a>
                </div>
            }
        </aside>
    })
}
