//! Offer image deletion route at `/profile/offers/{id}/images/delete`.

use haven_domain::image::ImageId;
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::Form, error::see_other},
};

#[derive(serde::Deserialize)]
pub struct DeleteOfferImageForm {
    pub image_id: uuid::Uuid,
}

#[topcoat::router::page([POST, DELETE])]
pub async fn delete_image(cx: &Cx, form: Option<Form<DeleteOfferImageForm>>) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let offer = crate::app::auth::guard::owned_offer(cx, offer_id).await?;

    let Some(form) = form else {
        return Err(see_other(format!("/profile/offers/{offer_id}")).into());
    };

    // Last photo protection: An offer needs at least one photo.
    if offer.images.len() <= 1 {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=last_image")).into());
    }

    let target_image_id = ImageId::from_uuid(form.0.image_id);

    let registry = crate::app::state::registry(cx);
    match registry
        .offer_images()
        .remove_image(offer_id, user.id, target_image_id)
        .await
    {
        Ok(_) => {}
        Err(haven_domain::ports::RepoError::Conflict { .. }) => {
            return Err(see_other(format!("/profile/offers/{offer_id}?error=last_image")).into());
        }
        Err(e) => return Err(crate::app::state::map_repo_err(e)),
    }

    Err(see_other(format!("/profile/offers/{offer_id}?image_deleted=1")).into())
}
