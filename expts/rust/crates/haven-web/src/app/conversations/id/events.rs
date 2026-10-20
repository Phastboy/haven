//! GET /conversations/{id}/events SSE handler

use crate::app::auth::guard::require_auth;
use crate::app::state::{map_repo_err, registry};
use futures_core::Stream;
use haven_domain::conversation::ConversationId;
use serde_json::json;
use std::time::Duration;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::content::sse::{Event, KeepAlive, Sse},
};

#[topcoat::router::route(GET)]
pub async fn conversation_events(
    cx: &Cx,
) -> TopcoatResult<Sse<impl Stream<Item = TopcoatResult<Event>> + use<>>> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let conv_id = ConversationId::from_uuid(id_uuid);
    let user = require_auth(cx).await?;
    let reg = registry(cx);

    let conv = reg
        .conversations()
        .find_by_id(conv_id)
        .await
        .map_err(map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    if !conv.pair.contains(user.id) {
        return Err(topcoat::router::error::not_found().into());
    }

    let mut rx = crate::app::state::broadcaster(cx).subscribe(conv_id);
    let current_user_id = user.id;

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            let is_own = msg.sender_id == current_user_id;
            let sender_name = if is_own {
                "You".to_string()
            } else {
                "Them".to_string()
            };

            let time_str = msg.created_at.format("%b %d, %H:%M").to_string();
            let msg_id = msg.id.to_string();
            let body_escaped = html_escape::encode_text(msg.body.as_str());

            let html = format!(
                r#"<div class="{}" id="msg-{}"><div class="message-meta"><span class="message-sender">{}</span><span class="message-time">{}</span></div><div class="message-body"><p>{}</p></div></div>"#,
                if is_own { "message-row own-message" } else { "message-row other-message" },
                msg_id,
                sender_name,
                time_str,
                body_escaped
            );

            let payload = json!({
                "id": msg_id,
                "html": html,
            });

            yield Ok(Event::new().id(&msg_id).data(payload.to_string()));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(20))))
}
