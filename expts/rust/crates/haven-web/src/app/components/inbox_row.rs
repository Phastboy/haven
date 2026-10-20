//! Inbox conversation row component.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

#[component]
pub async fn inbox_row(
    #[into] id: String,
    #[into] display_name: String,
    #[into] handle: String,
    #[default] avatar_url: Option<String>,
    #[into] last_message_body: String,
    #[into] last_message_time: String,
    #[default] is_unread: bool,
) -> TopcoatResult<impl View> {
    let initial = display_name
        .trim()
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string());

    let handle_formatted = if handle.starts_with('@') {
        handle
    } else {
        format!("@{handle}")
    };

    let href = format!("/conversations/{id}");

    Ok(view! {
        <a href=(href) class="inbox-row-link">
            <div class="inbox-row">
                <div class="inbox-avatar-slot" aria-hidden="true">
                    if let Some(url) = avatar_url {
                        <img src=(url) alt="" class="avatar-img" />
                    } else {
                        <span class="avatar-initial">(initial)</span>
                    }
                </div>
                <div class="inbox-content">
                    <div class="inbox-header">
                        <span class="inbox-name">(display_name)</span>
                        <span class="inbox-handle">(handle_formatted)</span>
                        <span class="inbox-time">(last_message_time)</span>
                    </div>
                    <div class="inbox-preview">
                        if is_unread {
                            <span class="unread-dot" aria-label="Unread message">"● "</span>
                        }
                        (last_message_body)
                    </div>
                </div>
            </div>
        </a>
    })
}
