//! Responsive, single-render offer photo gallery.

use topcoat::{
    Result as TopcoatResult,
    view::{View, component, view},
};

/// One prepared image in an offer gallery.
pub struct GalleryPhoto {
    /// Detail URL used by the first image and mobile slides.
    pub src: String,
    /// Detail 1x/2x srcset.
    pub srcset: String,
    /// Square card 1x/2x srcset.
    pub tile_srcset: String,
    /// Accessible image description.
    pub alt: String,
    /// Loading policy.
    pub loading: &'static str,
}

/// Gallery presentation inputs.
pub struct OfferGalleryData {
    /// Prepared offer photos, capped by domain to five.
    pub photos: Vec<GalleryPhoto>,
    /// Mobile save control, hidden at desktop widths.
    pub save: Option<crate::app::components::save_heart::SaveHeartData>,
}

/// Renders each photo exactly once; CSS assigns the responsive layout.
#[component]
pub async fn offer_gallery(data: OfferGalleryData) -> TopcoatResult<impl View> {
    let count = data.photos.len().clamp(1, 5);
    Ok(view! {
        if !data.photos.is_empty() {
            <div class=(format!("offer-gallery gallery-n{count}"))>
                for (index, photo) in data.photos.into_iter().enumerate() {
                    <div class=(format!("offer-gallery-photo gallery-photo-{}", index + 1))>
                        <picture>
                            <source media="(min-width: 641px)" srcset=(if index == 0 { photo.srcset.clone() } else { photo.tile_srcset.clone() }) />
                            <img src=(photo.src)
                                srcset=(photo.srcset)
                                alt=(photo.alt) class="offer-detail-image" width="640" height="480" loading=(photo.loading) />
                        </picture>
                        if index == 0 && count > 1 {
                            <span class="offer-gallery-count">(format!("{count} photos"))</span>
                        }
                        if index == 0 {
                            if let Some(save) = data.save.as_ref() {
                                <span class="offer-gallery-save">crate::app::components::save_heart::save_heart(props: crate::app::components::save_heart::SaveHeartData { slug: save.slug.clone(), is_saved: save.is_saved, return_to: None, show_label: false, is_authenticated: save.is_authenticated })</span>
                            }
                        }
                    </div>
                }
            </div>
        }
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "component test fixtures and assertions")]
mod tests {
    use super::*;
    use topcoat::{
        context::Cx,
        view::{ViewExt, view},
    };

    #[tokio::test]
    async fn gallery_emits_one_modifier_and_one_image_per_photo() {
        let cx = Cx::default();
        let __cx = &cx;
        for count in 1_usize..=5 {
            let photos = (0..count)
                .map(|index| GalleryPhoto {
                    src: format!("/detail-{index}.webp"),
                    srcset: format!("/detail-{index}.webp 1x, /detail-{index}-2x.webp 2x"),
                    tile_srcset: format!("/tile-{index}.webp 1x, /tile-{index}-2x.webp 2x"),
                    alt: format!("Offer, photo {} of {count}", index + 1),
                    loading: if index == 0 { "eager" } else { "lazy" },
                })
                .collect();
            let component = view! { offer_gallery(data: OfferGalleryData { photos, save: None }) };
            let html = component.single().await.expect("one gallery").render(__cx);
            assert!(html.contains(&format!("gallery-n{count}")));
            assert_eq!(html.matches("<img").count(), count);
            assert_eq!(html.contains(&format!("{count} photos")), count > 1);
            for index in 0..count {
                assert!(html.contains(&format!(
                    "src=\"/detail-{index}.webp\" srcset=\"/detail-{index}.webp 1x, /detail-{index}-2x.webp 2x\""
                )));
            }
        }
    }
}
