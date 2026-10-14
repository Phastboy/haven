#![allow(
    clippy::too_many_lines,
    reason = "Topcoat SSR view component with dialogs, form, and photo list"
)]

pub mod edit;
pub mod images;

use crate::app::components::button::{ButtonVariant, button_link};
use topcoat::{Result as TopcoatResult, context::Cx, router::error::see_other, view::view};

topcoat::router::module_param!(id: uuid::Uuid, error = not_found);

#[topcoat::router::query_params]
pub struct ViewOfferQuery {
    pub image_added: Option<String>,
    pub image_deleted: Option<String>,
    pub reordered: Option<String>,
    pub error: Option<String>,
}

#[topcoat::router::page]
pub async fn view_offer(cx: &Cx) -> TopcoatResult<impl topcoat::view::View> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer =
        crate::app::auth::guard::owned_offer(cx, haven_domain::offer::OfferId::from_uuid(id_uuid))
            .await?;

    let query = topcoat::router::query_params::<ViewOfferQuery>(cx).ok();
    let feedback_banner = if let Some(q) = query.as_ref() {
        if q.image_added.is_some() {
            Some(("success", "Photo added successfully."))
        } else if q.image_deleted.is_some() {
            Some(("success", "Photo removed."))
        } else if q.reordered.is_some() {
            Some(("success", "Photo order updated."))
        } else if let Some(err) = q.error.as_deref() {
            let msg = match err {
                "too_large" => "Photo is too large. Maximum size is 5 MB.",
                "invalid_format" => {
                    "Unsupported format. Only JPEG, PNG, and WebP are allowed (no SVG)."
                }
                "upload_failed" => "Photo upload failed. Please try again.",
                "max_images_reached" => {
                    "An offer can have at most 5 photos. Remove one to add another."
                }
                "last_image" => {
                    "An offer needs at least one photo. Add another first, or replace this one."
                }
                "empty_file" => "Selected file is empty. Please choose an image.",
                "missing_file" => "Please select an image file to upload.",
                _ => "An error occurred while managing photos.",
            };
            Some(("error", msg))
        } else {
            None
        }
    } else {
        None
    };

    let price_text = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };

    let is_public = offer.is_public();
    let public_slug_href = format!("/offers/{}", offer.slug);
    let edit_href = format!("/profile/offers/{}/edit", offer.id);
    let delete_action_href = format!("/profile/offers/{}", offer.id);
    let offer_title = offer.title.clone();

    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let total_images = offer.images.len();
    let upload_action_href = format!("/profile/offers/{}/images", offer.id);
    let delete_action_href_base = format!("/profile/offers/{}/images/delete", offer.id);
    let reorder_action_href = format!("/profile/offers/{}/images/reorder", offer.id);

    Ok(view! {
        <div class="offer-detail">
            <meta name="robots" content="noindex" />

            if let Some((kind, msg)) = feedback_banner {
                if kind == "success" {
                    <div class="notice-banner success-banner" role="status">
                        <p>(msg)</p>
                    </div>
                } else {
                    <div class="form-error-banner" role="alert">
                        <p>(msg)</p>
                    </div>
                }
            }

            if !is_public {
                <div class="notice-banner" role="status">
                    <p>"Only you can see this offer. Add a photo to make it public."</p>
                </div>
            }

            <div class="page-header">
                <div class="offer-detail-header">
                    <h1 class="offer-detail-title">(offer.title)</h1>
                    <div class="offer-detail-price">(price_text)</div>
                </div>
            </div>

            <div class="offer-detail-body">
                <p>(offer.description)</p>
            </div>

            <div class="owner-actions">
                button_link(
                    href: edit_href,
                    text: "Edit Offer",
                    variant: ButtonVariant::Primary,
                )
                button_link(
                    href: public_slug_href,
                    text: "View Public Page",
                    variant: ButtonVariant::Secondary,
                )
                <button
                    type="button"
                    popovertarget="delete-offer-dialog"
                    class="btn btn-danger"
                >
                    "Delete Offer"
                </button>
            </div>

            <dialog id="delete-offer-dialog" popover="auto" class="confirm-dialog">
                <div class="confirm-dialog-content">
                    <h2 class="dialog-title">"Delete Offer"</h2>
                    <p class="dialog-message">
                        "Are you sure you want to delete " <strong>(offer_title)</strong> "? This action cannot be undone."
                    </p>
                    <div class="dialog-actions">
                        <button
                            type="button"
                            popovertarget="delete-offer-dialog"
                            popovertargetaction="hide"
                            class="btn btn-secondary"
                        >
                            "Cancel"
                        </button>
                        <form method="post" action=(delete_action_href) class="dialog-confirm-form">
                            <button type="submit" class="btn btn-danger">
                                "Confirm Delete"
                            </button>
                        </form>
                    </div>
                </div>
            </dialog>

            <section class="offer-manage-photos">
                <div class="section-header">
                    <h2>"Photos"</h2>
                    <p class="text-muted">
                        if total_images == 0 {
                            "No photos yet. Add at least one photo to make this offer public."
                        } else {
                            (format!("{total_images} of 5 photos."))
                        }
                    </p>
                </div>

                if total_images > 0 {
                    <ul class="offer-photo-list" role="list">
                        for (idx, img) in offer.images.iter().enumerate() {
                            let img_id_str = img.id.to_string();
                            let delete_dlg_id = format!("del-img-{}", img.id.as_uuid());
                            <li class="offer-photo-row">
                                <div class="offer-photo-row-info">
                                    <img
                                        src=(crate::app::components::image::thumb_url(cloud_name, img.public_id.as_str()))
                                        srcset=(crate::app::components::image::thumb_srcset(cloud_name, img.public_id.as_str()))
                                        alt=""
                                        width="120"
                                        height="90"
                                        class="offer-manage-thumb"
                                    />
                                    <div>
                                        if idx == 0 {
                                            <span class="badge-cover">"Cover Photo"</span>
                                        } else {
                                            <span class="badge-subtle">(format!("Photo {}", idx + 1))</span>
                                        }
                                    </div>
                                </div>
                                <div class="offer-photo-controls">
                                    if total_images > 1 {
                                        if idx > 0 {
                                            <form method="post" action=(reorder_action_href.clone()) class="inline-form">
                                                <input type="hidden" name="image_id" value=(img_id_str.clone()) />
                                                <input type="hidden" name="direction" value="up" />
                                                <button type="submit" class="btn btn-secondary btn-sm">"Move up"</button>
                                            </form>
                                        }
                                        if idx + 1 < total_images {
                                            <form method="post" action=(reorder_action_href.clone()) class="inline-form">
                                                <input type="hidden" name="image_id" value=(img_id_str.clone()) />
                                                <input type="hidden" name="direction" value="down" />
                                                <button type="submit" class="btn btn-secondary btn-sm">"Move down"</button>
                                            </form>
                                        }
                                    }
                                    if total_images <= 1 {
                                        <p class="text-muted help-text">"An offer needs at least one photo. Add another first, or replace this one."</p>
                                    } else {
                                        <button
                                            type="button"
                                            popovertarget=(delete_dlg_id.clone())
                                            class="btn btn-danger btn-sm"
                                        >
                                            "Remove"
                                        </button>
                                        <dialog id=(delete_dlg_id.clone()) popover="auto" class="confirm-dialog">
                                            <div class="confirm-dialog-content">
                                                <h2 class="dialog-title">"Remove Photo"</h2>
                                                <p class="dialog-message">"Are you sure you want to remove this photo?"</p>
                                                <div class="dialog-actions">
                                                    <button
                                                        type="button"
                                                        popovertarget=(delete_dlg_id.clone())
                                                        popovertargetaction="hide"
                                                        class="btn btn-secondary"
                                                    >
                                                        "Cancel"
                                                    </button>
                                                    <form method="post" action=(delete_action_href_base.clone()) class="dialog-confirm-form">
                                                        <input type="hidden" name="image_id" value=(img_id_str.clone()) />
                                                        <button type="submit" class="btn btn-danger">
                                                            "Confirm Remove"
                                                        </button>
                                                    </form>
                                                </div>
                                            </div>
                                        </dialog>
                                    }
                                </div>
                            </li>
                        }
                    </ul>
                }

                if total_images >= 5 {
                    <p class="text-muted">"5 of 5 photos. Remove one to add another."</p>
                } else {
                    <form method="post" action=(upload_action_href) enctype="multipart/form-data" class="image-upload-form">
                        <div class="field">
                            <label for="offer-add-photo" class="field-label">"Add a photo"</label>
                            <input
                                type="file"
                                id="offer-add-photo"
                                name="image"
                                accept="image/jpeg,image/png,image/webp"
                                required="required"
                                class="file-input"
                            />
                            <p class="field-help text-muted">"JPEG, PNG, or WebP up to 5 MB. Aspect ratio 4:3 recommended."</p>
                        </div>
                        <button type="submit" class="btn btn-secondary">"Upload photo"</button>
                    </form>
                }
            </section>
        </div>
    })
}

#[topcoat::router::page([POST, DELETE])]
pub async fn delete_offer(cx: &Cx) -> TopcoatResult<()> {
    let id_uuid = *topcoat::router::path_param::<Id>(cx)?;
    let offer_id = haven_domain::offer::OfferId::from_uuid(id_uuid);

    // Authorization check
    let user = crate::app::auth::guard::require_owner_auth(cx).await?;

    crate::app::state::registry(cx)
        .offers()
        .delete(offer_id, user.id)
        .await
        .map_err(crate::app::state::map_repo_err)?;

    Err(see_other("/profile/offers").into())
}
