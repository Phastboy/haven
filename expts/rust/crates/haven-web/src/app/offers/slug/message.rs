//! POST /offers/{slug}/message handler for inquiries

use topcoat::{
    Result as TopcoatResult, context::Cx, router::content::Form, router::error::see_other,
};

use crate::app::auth::guard::require_auth;
use crate::app::state::{map_repo_err, registry};
use haven_domain::conversation::{ConversationPair, MessageBody, NewMessage};

#[derive(serde::Deserialize)]
pub struct OfferInquiryForm {
    pub body: String,
}

#[topcoat::router::page(POST)]
pub async fn send_offer_inquiry(cx: &Cx, form: Form<OfferInquiryForm>) -> TopcoatResult<()> {
    let slug_raw: &str = topcoat::router::path_param::<super::Slug>(cx);
    let offer_slug = haven_domain::offer::OfferSlug::parse(slug_raw)
        .map_err(|_| topcoat::router::error::not_found())?;

    let user = require_auth(cx).await?;
    let reg = registry(cx);

    let offer = reg
        .offers()
        .find_by_slug(&offer_slug)
        .await
        .map_err(map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;

    // Reject self-messaging
    if offer.user_id == user.id {
        return Err(topcoat::router::error::bad_request("cannot message your own offer").into());
    }

    let parsed_body = MessageBody::parse(&form.0.body)
        .map_err(|e| topcoat::router::error::bad_request(e.to_string()))?;

    let pair = ConversationPair::new(user.id, offer.user_id)
        .map_err(|e| topcoat::router::error::bad_request(e.to_string()))?;

    // Get or create conversation for the pair
    let conv = reg
        .conversations()
        .get_or_create(pair)
        .await
        .map_err(map_repo_err)?;

    // Append inquiry message tagged with this offer
    let msg = reg
        .conversations()
        .append(
            conv.id,
            NewMessage {
                sender_id: user.id,
                body: parsed_body,
                offer_id: Some(offer.id),
            },
        )
        .await
        .map_err(map_repo_err)?;

    // Broadcast
    crate::app::state::broadcaster(cx).broadcast(&msg);

    Err(see_other(format!("/conversations/{}", conv.id)).into())
}
