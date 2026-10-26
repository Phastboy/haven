use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

pub struct MobileBackBarData {
    pub href: String,
}

#[component]
pub async fn mobile_back_bar(props: MobileBackBarData) -> TopcoatResult<impl View> {
    Ok(view! {
        <header class="app-mobile-back-bar">
            <a href=(props.href) class="app-mobile-back-link">
                <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><path d="m12 19-7-7 7-7"/></svg>
                <span>"Back"</span>
            </a>
        </header>
    })
}
