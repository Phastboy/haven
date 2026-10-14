//! Profile avatar upload route at `/profile/avatar`.

pub mod delete;

use haven_domain::image::{ImagePublicId, MAX_IMAGE_BYTES, validate_image_magic_bytes};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::{content::multipart::Multipart, error::see_other},
};

#[topcoat::router::page(POST)]
pub async fn upload_avatar(cx: &Cx, multipart: Option<Multipart>) -> TopcoatResult<()> {
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    let Some(mut multipart) = multipart else {
        return Err(see_other("/profile/edit?error=missing_file").into());
    };

    let mut avatar_bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("avatar") {
            let Ok(data) = field.bytes().await else {
                return Err(see_other("/profile/edit?error=too_large").into());
            };
            if !data.is_empty() {
                avatar_bytes = Some(data.to_vec());
                break;
            }
        }
    }

    let Some(bytes) = avatar_bytes else {
        return Err(see_other("/profile/edit?error=empty_file").into());
    };

    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(see_other("/profile/edit?error=too_large").into());
    }

    if validate_image_magic_bytes(&bytes).is_err() {
        return Err(see_other("/profile/edit?error=invalid_format").into());
    }

    let storage = crate::app::state::image_storage(cx);
    let folder = format!("users/{}", user.id);
    let suffix = uuid::Uuid::new_v4().to_string();

    let Ok(stored) = storage.upload(&folder, &suffix, &bytes).await else {
        return Err(see_other("/profile/edit?error=upload_failed").into());
    };

    let Ok(public_id) = ImagePublicId::parse(&stored.public_id) else {
        return Err(see_other("/profile/edit?error=upload_failed").into());
    };

    let registry = crate::app::state::registry(cx);
    registry
        .users()
        .update_avatar(user.id, Some(&public_id))
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other("/profile/edit?avatar_updated=1").into())
}
