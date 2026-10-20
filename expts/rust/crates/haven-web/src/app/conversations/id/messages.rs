//! POST /conversations/{id}/messages handler

use topcoat::{
    Result as TopcoatResult, context::Cx, router::content::Form, router::error::see_other,
};
use uuid::Uuid;

use crate::app::auth::guard::require_auth;
use crate::app::state::{map_repo_err, registry};
use haven_domain::conversation::{ConversationId, MessageBody, NewMessage};
use haven_domain::offer::OfferId;

#[derive(serde::Deserialize)]
pub struct SendMessageForm {
    pub body: String,
    pub offer_id: Option<String>,
}

#[topcoat::router::page(POST)]
pub async fn send_message(cx: &Cx, form: Form<SendMessageForm>) -> TopcoatResult<()> {
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

    let parsed_body = MessageBody::parse(&form.0.body)
        .map_err(|e| topcoat::router::error::bad_request(e.to_string()))?;

    let offer_id = if let Some(oid_str) = &form.0.offer_id {
        let trimmed = oid_str.trim();
        if trimmed.is_empty() {
            None
        } else {
            let parsed_uuid = Uuid::parse_str(trimmed)
                .map_err(|_| topcoat::router::error::bad_request("invalid offer id"))?;
            let oid = OfferId::from_uuid(parsed_uuid);

            // Verify offer exists and belongs to either participant
            let owned_by_sender = reg.offers().find_owned(oid, user.id).await.unwrap_or(None);
            let Some(other_user_id) = conv.pair.other(user.id) else {
                return Err(topcoat::router::error::not_found().into());
            };
            let owned_by_other = reg
                .offers()
                .find_owned(oid, other_user_id)
                .await
                .unwrap_or(None);

            if owned_by_sender.is_none() && owned_by_other.is_none() {
                return Err(topcoat::router::error::bad_request("invalid offer tag").into());
            }

            Some(oid)
        }
    } else {
        None
    };

    let msg = reg
        .conversations()
        .append(
            conv_id,
            NewMessage {
                sender_id: user.id,
                body: parsed_body,
                offer_id,
            },
        )
        .await
        .map_err(map_repo_err)?;

    // Broadcast to live listeners
    crate::app::state::broadcaster(cx).broadcast(&msg);

    Err(see_other(format!("/conversations/{conv_id}")).into())
}
