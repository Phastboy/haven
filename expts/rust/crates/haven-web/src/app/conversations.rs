//! Conversations module routes and handlers.

pub mod id;

#[cfg(test)]
mod tests;

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};

use crate::app::auth::guard::require_auth;
use crate::app::components::empty_state::empty_state;
use crate::app::components::inbox_row::inbox_row;
use crate::app::components::pagination::pagination_nav;
use crate::app::state::{map_repo_err, registry};

struct InboxRowData {
    pub id: String,
    pub display_name: String,
    pub handle: String,
    pub avatar_url: Option<String>,
    pub last_message_body: String,
    pub last_message_time: String,
    pub is_unread: bool,
}

#[topcoat::router::query_params]
pub struct InboxQuery {
    pub cursor: Option<String>,
}

#[topcoat::router::page]
pub async fn inbox_page(cx: &Cx) -> TopcoatResult<impl View> {
    let user = require_auth(cx).await?;
    let query = topcoat::router::query_params::<InboxQuery>(cx).ok();
    let cursor_param = match query.as_ref().and_then(|q| q.cursor.as_deref()) {
        Some(c) => {
            haven_domain::conversation::ConversationCursor::decode(c)
                .map_err(|_| topcoat::router::error::bad_request("invalid cursor"))?;
            Some(c)
        }
        None => None,
    };

    let page = registry(cx)
        .conversations()
        .list_for_user(user.id, cursor_param, 20)
        .await
        .map_err(map_repo_err)?;

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let items_count = page.items.len();

    let items_view_data: Vec<InboxRowData> = page
        .items
        .into_iter()
        .map(|it| {
            let avatar_url = it
                .other_avatar_public_id
                .map(|pid| crate::app::components::image::avatar_url(cloud_name, pid.as_str()));

            InboxRowData {
                id: it.id.to_string(),
                display_name: it.other_display_name.as_str().to_string(),
                handle: it.other_handle.as_str().to_string(),
                avatar_url,
                last_message_body: it.last_message_body,
                last_message_time: it.last_message_at.format("%b %d, %H:%M").to_string(),
                is_unread: it.is_unread,
            }
        })
        .collect();

    Ok(view! {
        <div class="conversations-inbox">
            <meta name="robots" content="noindex" />
            <h1 class="page-title">"Messages"</h1>

            if items_count == 0 && cursor_param.is_none() {
                empty_state(
                    title: "No messages yet",
                    message: "When you message someone about an offer or someone reaches out to you, conversations will appear here.",
                )
            } else {
                <div class="inbox-list">
                    for it in items_view_data {
                        inbox_row(
                            id: it.id,
                            display_name: it.display_name,
                            handle: it.handle,
                            avatar_url: it.avatar_url,
                            last_message_body: it.last_message_body,
                            last_message_time: it.last_message_time,
                            is_unread: it.is_unread,
                        )
                    }
                </div>
                if let Some(next) = page.next_cursor {
                    pagination_nav(next_href: Some(format!("/conversations?cursor={next}")))
                }
            }
        </div>
    })
}
