//! Offer image upload route at `/profile/offers/{id}/images`.

pub mod delete;
pub mod reorder;

use haven_domain::image::{ImageDimensions, ImagePublicId, MAX_IMAGE_BYTES, validate_image_magic_bytes};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::multipart::Multipart, error::see_other},
};

#[topcoat::router::page(POST)]
pub async fn upload_image(cx: &Cx, multipart: Option<Multipart>) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<super::Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    let user = crate::app::auth::guard::require_owner_auth(cx).await?;
    let offer = crate::app::auth::guard::owned_offer(cx, offer_id).await?;

    if offer.images.len() >= 5 {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=max_images_reached")).into());
    }

    let Some(mut multipart) = multipart else {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=missing_file")).into());
    };

    let mut image_bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("image") {
            let Ok(data) = field.bytes().await else {
                return Err(see_other(format!("/profile/offers/{offer_id}?error=too_large")).into());
            };
            if !data.is_empty() {
                image_bytes = Some(data.to_vec());
                break;
            }
        }
    }

    let Some(bytes) = image_bytes else {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=empty_file")).into());
    };

    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=too_large")).into());
    }

    if validate_image_magic_bytes(&bytes).is_err() {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=invalid_format")).into());
    }

    let storage = crate::app::state::image_storage(cx);
    let folder = format!("offers/{offer_id}");
    let suffix = uuid::Uuid::new_v4().to_string();

    let Ok(stored) = storage.upload(&folder, &suffix, &bytes).await else {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=upload_failed")).into());
    };

    let Ok(public_id) = ImagePublicId::parse(&stored.public_id) else {
        return Err(see_other(format!("/profile/offers/{offer_id}?error=upload_failed")).into());
    };

    let dimensions = ImageDimensions::new(stored.width, stored.height).unwrap_or_default();

    let registry = crate::app::state::registry(cx);
    registry
        .offer_images()
        .add_image(offer_id, user.id, &public_id, dimensions)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other(format!("/profile/offers/{offer_id}?image_added=1")).into())
}
