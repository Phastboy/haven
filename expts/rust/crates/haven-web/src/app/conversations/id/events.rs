//! GET /conversations/{id}/events SSE handler

use crate::app::auth::guard::require_auth;
use crate::app::state::{AppState, map_repo_err, registry};
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

    let other_user_id = conv.pair.other(user.id);
    let other_name = match other_user_id {
        Some(oid) => reg
            .users()
            .find_by_id(oid)
            .await
            .ok()
            .flatten()
            .map(|u| u.display_name.as_str().to_string()),
        None => None,
    }
    .unwrap_or_else(|| "Them".to_string());

    let app_st = topcoat::context::app_context::<AppState>(cx);
    let reg_arc = app_st.registry.clone();
    let cloud_name = app_st.cloudinary_cloud_name.clone();
    let mut rx = app_st.broadcaster.subscribe(conv_id);
    let current_user_id = user.id;

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            let is_own = msg.sender_id == current_user_id;
            let sender_name = if is_own {
                "You".to_string()
            } else {
                other_name.clone()
            };
            let sender_name_escaped = html_escape::encode_text(&sender_name);

            let time_str = msg.created_at.format("%b %d, %H:%M").to_string();
            let msg_id = msg.id.to_string();
            let body_escaped = html_escape::encode_text(msg.body.as_str());

            let offer_tag_html = if let Some(oid) = msg.offer_id {
                let maybe_off = if let Some(off) = reg_arc
                    .offers()
                    .find_owned(oid, msg.sender_id)
                    .await
                    .unwrap_or(None)
                {
                    Some(off)
                } else {
                    let other_id = if msg.sender_id == current_user_id {
                        other_user_id.unwrap_or(current_user_id)
                    } else {
                        current_user_id
                    };
                    reg_arc.offers().find_owned(oid, other_id).await.unwrap_or(None)
                };
                if let Some(off) = maybe_off {
                    let cover = off.images.first().map(|img| {
                        crate::app::components::image::thumb_url(cloud_name.as_deref(), img.public_id.as_str())
                    });
                    let title_esc = html_escape::encode_text(off.title.as_str());
                    let slug_esc = html_escape::encode_text(off.slug.as_str());
                    let thumb_html = if let Some(cover_url) = cover {
                        let cover_esc = html_escape::encode_text(&cover_url);
                        format!(r#"<img src="{cover_esc}" alt="" class="offer-tag-thumb" width="48" height="48" />"#)
                    } else {
                        String::new()
                    };
                    format!(
                        r#"<div class="message-offer-tag"><a href="/offers/{slug_esc}" class="offer-tag-link">{thumb_html}<span class="offer-tag-title">{title_esc}</span></a></div>"#
                    )
                } else {
                    r#"<div class="message-offer-tag"><span class="offer-tag-unavailable">This offer is no longer available</span></div>"#.to_string()
                }
            } else {
                String::new()
            };

            let row_class = if is_own {
                "message-row own-message"
            } else {
                "message-row other-message"
            };

            let html = format!(
                r#"<div class="{row_class}" id="msg-{msg_id}"><div class="message-meta"><span class="message-sender">{sender_name_escaped}</span><span class="message-time">{time_str}</span></div>{offer_tag_html}<div class="message-body"><p>{body_escaped}</p></div></div>"#
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
