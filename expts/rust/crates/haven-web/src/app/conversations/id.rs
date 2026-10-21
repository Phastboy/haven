//! Single conversation view, message send, and SSE stream.

#![allow(
    clippy::too_many_lines,
    clippy::let_underscore_must_use,
    reason = "Conversation thread view with SSR components"
)]

pub mod events;
pub mod messages;

use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};
use uuid::Uuid;

use crate::app::auth::guard::require_auth;
use crate::app::components::message_item::message_item;
use crate::app::state::{map_repo_err, registry};
use haven_domain::conversation::ConversationId;

topcoat::router::module_param!(id: Uuid, error = not_found);

#[topcoat::router::query_params]
pub struct ThreadQuery {
    pub cursor: Option<String>,
}

#[topcoat::router::page]
pub async fn thread_page(cx: &Cx) -> TopcoatResult<impl View> {
    let conv_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let conv_id = ConversationId::from_uuid(conv_uuid);
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

    // Mark as read immediately on viewing
    let _ = reg.conversations().mark_read(conv_id, user.id).await;

    let other_user_id = conv
        .pair
        .other(user.id)
        .ok_or_else(topcoat::router::error::not_found)?;
    let other_user = reg
        .users()
        .find_by_id(other_user_id)
        .await
        .map_err(map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    let query = topcoat::router::query_params::<ThreadQuery>(cx).ok();
    let cursor_param = match query.as_ref().and_then(|q| q.cursor.as_deref()) {
        Some(c) => {
            haven_domain::conversation::MessageCursor::decode(c)
                .map_err(|_| topcoat::router::error::bad_request("invalid cursor"))?;
            Some(c)
        }
        None => None,
    };

    let msgs_page = reg
        .conversations()
        .messages(conv_id, cursor_param, 50)
        .await
        .map_err(map_repo_err)?;

    let user_offers = reg.offers().find_by_user(user.id).await.unwrap_or_default();

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);

    let mut offer_cache: std::collections::HashMap<
        haven_domain::offer::OfferId,
        Option<OfferTagData>,
    > = std::collections::HashMap::new();
    let mut message_items = Vec::with_capacity(msgs_page.items.len());
    for m in &msgs_page.items {
        let is_own = m.sender_id == user.id;
        let sender_name = if is_own {
            "You".to_string()
        } else {
            other_user.display_name.as_str().to_string()
        };

        let offer_tag = if let Some(oid) = m.offer_id {
            if let Some(cached) = offer_cache.get(&oid) {
                cached.clone()
            } else {
                let maybe_off = if let Some(off) = reg
                    .offers()
                    .find_owned(oid, m.sender_id)
                    .await
                    .unwrap_or(None)
                {
                    Some(off)
                } else {
                    let other_id = if m.sender_id == user.id {
                        other_user_id
                    } else {
                        user.id
                    };
                    reg.offers().find_owned(oid, other_id).await.unwrap_or(None)
                };
                let tag = if let Some(off) = maybe_off {
                    let cover = off.images.first().map(|img| {
                        crate::app::components::image::thumb_url(cloud_name, img.public_id.as_str())
                    });
                    Some(OfferTagData {
                        title: off.title,
                        slug: off.slug.as_str().to_string(),
                        cover_url: cover,
                        is_available: true,
                    })
                } else {
                    Some(OfferTagData {
                        title: String::new(),
                        slug: String::new(),
                        cover_url: None,
                        is_available: false,
                    })
                };
                offer_cache.insert(oid, tag.clone());
                tag
            }
        } else {
            None
        };

        message_items.push(MessageItemData {
            id: m.id.to_string(),
            is_own,
            sender_name,
            body: m.body.as_str().to_string(),
            time_str: m.created_at.format("%b %d, %H:%M").to_string(),
            offer_tag,
        });
    }

    let other_name = other_user.display_name.as_str().to_string();
    let other_handle = format!("@{}", other_user.handle.as_str());
    let conv_id_str = conv_id.to_string();
    let last_msg_id = msgs_page
        .items
        .last()
        .map(|m| m.id.to_string())
        .unwrap_or_default();

    Ok(view! {
        <div class="conversation-thread" data-conversation-id=(conv_id_str.clone()) data-last-message-id=(last_msg_id.clone())>
            <meta name="robots" content="noindex" />
            <header class="thread-header">
                <div class="thread-recipient">
                    <span class="thread-recipient-name">(other_name)</span>
                    <span class="thread-recipient-handle">(other_handle)</span>
                </div>
            </header>

            <div class="thread-messages" id="thread-messages">
                if let Some(next) = &msgs_page.next_cursor {
                    <div class="thread-pagination">
                        <a href=(format!("/conversations/{conv_id_str}?cursor={next}")) class="btn btn-secondary">"← Earlier messages"</a>
                    </div>
                }
                for item in message_items {
                    message_item(
                        id: item.id,
                        is_own: item.is_own,
                        sender_name: item.sender_name,
                        body: item.body,
                        time_str: item.time_str,
                        offer_tag_title: item.offer_tag.as_ref().map(|t| t.title.clone()),
                        offer_tag_slug: item.offer_tag.as_ref().map(|t| t.slug.clone()),
                        offer_tag_cover: item.offer_tag.as_ref().and_then(|t| t.cover_url.clone()),
                        offer_tag_available: item.offer_tag.as_ref().is_some_and(|t| t.is_available),
                    )
                }
            </div>

            <form action=(format!("/conversations/{conv_id_str}/messages")) method="post" class="thread-composer">
                if !user_offers.is_empty() {
                    <div class="composer-offer-select">
                        <label for="offer_id">"Attach one of your offers (optional):"</label>
                        <select name="offer_id" id="offer_id">
                            <option value="">"None"</option>
                            for o in user_offers {
                                <option value=(o.id.to_string())>(o.title)</option>
                            }
                        </select>
                    </div>
                }
                <div class="composer-input-row">
                    <textarea name="body" placeholder="Write a message..." rows="3" required="required" maxlength="2000"></textarea>
                    <button type="submit" class="btn btn-primary">"Send"</button>
                </div>
            </form>

            <script>
                (r"
                (function() {
                    var container = document.getElementById('thread-messages');
                    if (container) {
                        container.scrollTop = container.scrollHeight;
                    }
                    if (window.EventSource) {
                        var convId = document.querySelector('.conversation-thread').getAttribute('data-conversation-id');
                        var lastMsgId = document.querySelector('.conversation-thread').getAttribute('data-last-message-id');
                        var url = '/conversations/' + convId + '/events';
                        if (lastMsgId) {
                            url += '?last_event_id=' + encodeURIComponent(lastMsgId);
                        }
                        var es = new EventSource(url);
                        es.addEventListener('message', function(e) {
                            try {
                                var data = JSON.parse(e.data);
                                if (data && data.html) {
                                    var temp = document.createElement('div');
                                    temp.innerHTML = data.html;
                                    var msgEl = temp.firstElementChild;
                                    if (msgEl && !document.getElementById(msgEl.id)) {
                                        container.appendChild(msgEl);
                                        container.scrollTop = container.scrollHeight;
                                    }
                                }
                            } catch (err) {}
                        });
                    }
                })();
                ")
            </script>
        </div>
    })
}

struct MessageItemData {
    pub id: String,
    pub is_own: bool,
    pub sender_name: String,
    pub body: String,
    pub time_str: String,
    pub offer_tag: Option<OfferTagData>,
}

#[derive(Clone)]
struct OfferTagData {
    pub title: String,
    pub slug: String,
    pub cover_url: Option<String>,
    pub is_available: bool,
}
