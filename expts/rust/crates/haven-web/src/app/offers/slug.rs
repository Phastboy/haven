//! Public offer detail route: `/offers/{slug}`.

pub mod action_card;
pub mod cache;
pub mod gallery;
pub mod maker_row;
pub mod message;
pub mod save;
pub mod viewer;

use action_card::{ActionCardData, action_card as render_action_card};
use cache::{matches_if_none_match, offer_etag};
use gallery::{GalleryPhoto, OfferGalleryData, offer_gallery as render_offer_gallery};
use maker_row::{MakerRowData, maker_row as render_maker_row};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    view::{View, view},
};
use viewer::{OfferViewer, OfferViewerInput, offer_viewer};

topcoat::router::module_param!(slug);

#[topcoat::router::page]
pub async fn view_slug(cx: &Cx) -> TopcoatResult<impl View> {
    let slug_raw: &str = topcoat::router::path_param::<Slug>(cx);
    let slug = haven_domain::offer::OfferSlug::parse(slug_raw)
        .map_err(|_| topcoat::router::error::not_found())?;
    let registry = crate::app::state::registry(cx);
    let offer = registry
        .offers()
        .find_by_slug(&slug)
        .await
        .map_err(crate::app::state::map_repo_err)?
        .ok_or_else(topcoat::router::error::not_found)?;
    let person = crate::app::auth::guard::current_user(cx).await?;
    let is_owner = person.as_ref().is_some_and(|p| p.id == offer.user_id);
    if !offer.is_public() && !is_owner {
        return Err(topcoat::router::error::not_found().into());
    }

    let saved = if offer.is_public() {
        if let Some(visitor) = person.as_ref().filter(|p| p.id != offer.user_id) {
            registry
                .saved_offers()
                .is_saved(visitor.id, offer.id)
                .await
                .map_err(crate::app::state::map_repo_err)?
        } else {
            false
        }
    } else {
        false
    };
    let role = offer_viewer(OfferViewerInput {
        viewer_id: person.as_ref().map(|p| p.id),
        owner_id: offer.user_id,
        saved,
    });

    let maker = registry
        .users()
        .find_by_id(offer.user_id)
        .await
        .map_err(crate::app::state::map_repo_err)?;
    let maker_name = maker.as_ref().map_or_else(
        || "Maker".to_string(),
        |m| m.display_name.as_str().to_string(),
    );
    let maker_updated_at_micros = maker
        .as_ref()
        .map_or(0, |user| user.updated_at.timestamp_micros());
    let maker_row_data = maker.map_or(
        MakerRowData {
            display_name: "Maker".to_string(),
            profile_href: "/offers".to_string(),
            joined_date: String::new(),
            avatar_url: None,
            avatar_srcset: None,
            initial: "M".to_string(),
        },
        |user| {
            let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
            let avatar = user.avatar_public_id.as_ref().map(|id| {
                (
                    crate::app::components::image::avatar_url(cloud_name, id.as_str()),
                    crate::app::components::image::avatar_srcset(cloud_name, id.as_str()),
                )
            });
            MakerRowData {
                display_name: user.display_name.as_str().to_string(),
                profile_href: format!("/profiles/{}", user.handle.as_str()),
                joined_date: crate::app::components::date::joined_date(&user.created_at),
                avatar_url: avatar.as_ref().map(|a| a.0.clone()),
                avatar_srcset: avatar.map(|a| a.1),
                initial: user
                    .display_name
                    .as_str()
                    .chars()
                    .next()
                    .unwrap_or('M')
                    .to_uppercase()
                    .to_string(),
            }
        },
    );

    let etag = offer_etag(
        offer.id.as_uuid().as_u128(),
        offer.updated_at.timestamp_micros(),
        maker_updated_at_micros,
    );
    let headers = topcoat::router::response::response_headers(cx);
    if person.is_some() {
        headers.append(
            topcoat::router::header::HeaderName::from_static("cache-control"),
            topcoat::router::header::HeaderValue::from_static("private, no-store"),
        );
    } else {
        headers.append(
            topcoat::router::header::HeaderName::from_static("cache-control"),
            topcoat::router::header::HeaderValue::from_static("public, max-age=60, s-maxage=300"),
        );
        if let Ok(value) = topcoat::router::header::HeaderValue::from_str(&etag) {
            headers.append(
                topcoat::router::header::HeaderName::from_static("etag"),
                value,
            );
        }
    }
    let not_modified = person.is_none()
        && topcoat::router::request::headers(cx)
            .get_all(http::header::IF_NONE_MATCH)
            .iter()
            .any(|header| {
                header
                    .to_str()
                    .is_ok_and(|tag| matches_if_none_match(tag, &etag))
            });

    let price = if offer.price.as_i32() == 0 {
        "Free".to_string()
    } else {
        format!("{} {}", offer.price.as_i32(), offer.currency.as_str())
    };
    let title = offer.title.clone();
    let slug_text = offer.slug.as_str().to_string();
    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let photos = offer
        .images
        .iter()
        .enumerate()
        .map(|(index, image)| GalleryPhoto {
            src: crate::app::components::image::detail_url(cloud_name, image.public_id.as_str()),
            srcset: crate::app::components::image::detail_srcset(
                cloud_name,
                image.public_id.as_str(),
            ),
            tile_srcset: crate::app::components::image::card_sq_srcset(
                cloud_name,
                image.public_id.as_str(),
            ),
            alt: format!(
                "{}, photo {} of {}",
                offer.title,
                index.saturating_add(1),
                offer.images.len()
            ),
            loading: if index == 0 { "eager" } else { "lazy" },
        })
        .collect();
    let gallery_save =
        matches!(role, OfferViewer::Visitor { .. } | OfferViewer::Anonymous).then(|| {
            crate::app::components::save_heart::SaveHeartData {
                slug: slug.clone(),
                is_saved: matches!(role, OfferViewer::Visitor { saved: true }),
                return_to: None,
                show_label: false,
                is_authenticated: !matches!(role, OfferViewer::Anonymous),
            }
        });
    let gallery_data = OfferGalleryData {
        photos,
        save: gallery_save,
    };
    let action_data = ActionCardData {
        slug: slug_text.clone(),
        price: price.clone(),
        maker_name,
        role,
    };
    let save_slug = slug.clone();

    Ok(view! {
        if not_modified { (http::StatusCode::NOT_MODIFIED) } else {
            <div id=(format!("offer-{}", slug_text)) class="offer-detail">
                <link rel="canonical" href=(format!("/offers/{slug_text}")) />
                <div class="offer-detail-back-slot"></div>
                <section class="offer-title-row">
                    <h1 class="offer-detail-title">(title)</h1>
                    if matches!(role, OfferViewer::Visitor { .. } | OfferViewer::Anonymous) {
                        <div class="offer-title-save"><div class="save-heart-desktop">
                            crate::app::components::save_heart::save_heart(props: crate::app::components::save_heart::SaveHeartData {
                                slug: save_slug.clone(), is_saved: matches!(role, OfferViewer::Visitor { saved: true }), return_to: None, show_label: true, is_authenticated: !matches!(role, OfferViewer::Anonymous),
                            })
                        </div></div>
                    }
                </section>
                <div class="offer-detail-price">(price)</div>
                render_offer_gallery(data: gallery_data)
                <div class="offer-detail-columns">
                    <div class="offer-detail-copy">
                        render_maker_row(data: maker_row_data)
                        <div class="offer-detail-body">(offer.description)</div>
                    </div>
                    <aside class="offer-action-column">render_action_card(data: action_data)</aside>
                </div>
                match role {
                    OfferViewer::Visitor { .. } => <div class="offer-mobile-bar"><a class="offer-primary-button" href="#message">"Message"</a></div>,
                    OfferViewer::Anonymous => <div class="offer-mobile-bar"><a class="offer-primary-button" href="/auth/sign-in">"Sign in to message"</a></div>,
                    OfferViewer::Owner => {},
                }
            </div>
        }
    })
}
