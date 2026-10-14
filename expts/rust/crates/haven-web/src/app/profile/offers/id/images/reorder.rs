//! Offer image reordering route at `/profile/offers/{id}/images/reorder`.

use haven_domain::image::ImageId;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::Form, error::see_other},
};

#[derive(serde::Deserialize)]
pub struct ReorderOfferImageForm {
    pub image_id: uuid::Uuid,
    pub direction: String,
}

#[topcoat::router::page(POST)]
pub async fn reorder_image(
    cx: &Cx,
    form: Option<Form<ReorderOfferImageForm>>,
) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let offer = crate::app::auth::guard::owned_offer(cx, offer_id).await?;

    let Some(form) = form else {
        return Err(see_other(format!("/profile/offers/{offer_id}")).into());
    };

    let target_image_id = ImageId::from_uuid(form.0.image_id);
    let direction = form.0.direction.trim().to_lowercase();

    let mut current_order: Vec<ImageId> = offer.images.iter().map(|img| img.id).collect();
    let Some(idx) = current_order.iter().position(|id| *id == target_image_id) else {
        return Err(see_other(format!("/profile/offers/{offer_id}")).into());
    };

    let changed = match direction.as_str() {
        "up" => {
            if let Some(prev) = idx.checked_sub(1) {
                current_order.swap(idx, prev);
                true
            } else {
                false
            }
        }
        "down" => {
            if let Some(next) = idx.checked_add(1) {
                if next < current_order.len() {
                    current_order.swap(idx, next);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    };

    if changed {
        let registry = crate::app::state::registry(cx);
        registry
            .offer_images()
            .reorder_images(offer_id, user.id, &current_order)
            .await
            .map_err(crate::app::state::map_repo_err)?;
    }

    Err(see_other(format!("/profile/offers/{offer_id}?reordered=1")).into())
}
