//! Centralized Cloudinary image URL and srcset builder.
//!
//! Generates CDN delivery URLs with Cloudinary named transformations adhering
//! to the strict transformations budget defense in `haven-ui`.

/// Image transformation variants supported by Haven.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageVariant {
    /// Profile avatar: 1:1 square aspect ratio (96x96 at 1x, 192x192 at 2x).
    Avatar,
    /// Management list / scanning thumbnail: 4:3 aspect ratio (120x90 at 1x, 240x180 at 2x).
    Thumb,
    /// Public discovery card cover: 4:3 aspect ratio (320x240 at 1x, 640x480 at 2x).
    Card,
    /// Square public discovery card cover.
    CardSquare,
    /// Public offer detail photo view: 4:3 aspect ratio (640x480 at 1x, 960x720 at 2x).
    Detail,
}

/// Device pixel ratio (DPR) resolution multipliers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageDpr {
    /// Standard density display (1x).
    Dpr1x,
    /// High density Retina / modern mobile display (2x).
    Dpr2x,
}

impl ImageVariant {
    /// Returns the exact Cloudinary named transformation identifier.
    #[must_use]
    pub fn transformation_name(self, dpr: ImageDpr) -> &'static str {
        match (self, dpr) {
            (Self::Avatar, ImageDpr::Dpr1x) => "t_avatar",
            (Self::Avatar, ImageDpr::Dpr2x) => "t_avatar_2x",
            (Self::Thumb, ImageDpr::Dpr1x) => "t_thumb",
            (Self::Thumb, ImageDpr::Dpr2x) => "t_thumb_2x",
            (Self::Card, ImageDpr::Dpr1x) => "t_card",
            (Self::Card, ImageDpr::Dpr2x) => "t_card_2x",
            (Self::CardSquare, ImageDpr::Dpr1x) => "t_card_sq",
            (Self::CardSquare, ImageDpr::Dpr2x) => "t_card_sq_2x",
            (Self::Detail, ImageDpr::Dpr1x) => "t_detail",
            (Self::Detail, ImageDpr::Dpr2x) => "t_detail_2x",
        }
    }

    /// Returns the pixel width and height (width, height) for this variant and DPR.
    #[must_use]
    pub fn dimensions(self, dpr: ImageDpr) -> (u32, u32) {
        match (self, dpr) {
            (Self::Avatar, ImageDpr::Dpr1x) => (96, 96),
            (Self::Avatar, ImageDpr::Dpr2x) => (192, 192),
            (Self::Thumb, ImageDpr::Dpr1x) => (120, 90),
            (Self::Thumb, ImageDpr::Dpr2x) => (240, 180),
            (Self::Card, ImageDpr::Dpr1x) => (320, 240),
            (Self::CardSquare, ImageDpr::Dpr1x) => (320, 320),
            (Self::CardSquare, ImageDpr::Dpr2x) => (640, 640),
            (Self::Card, ImageDpr::Dpr2x) | (Self::Detail, ImageDpr::Dpr1x) => (640, 480),
            (Self::Detail, ImageDpr::Dpr2x) => (960, 720),
        }
    }
}

/// Default placeholder cloud name used when Cloudinary is not configured in environment.
pub const DEFAULT_FALLBACK_CLOUD_NAME: &str = "haven";

/// Builds a delivery URL for a specific image variant and resolution multiplier.
///
/// Images are served from Cloudinary's private asset delivery path using named transformations.
#[must_use]
#[allow(
    clippy::too_many_arguments,
    reason = "Image URL builder takes cloud name, public ID, variant, and resolution multiplier"
)]
pub fn build_image_url(
    cloud_name: Option<&str>,
    public_id: &str,
    variant: ImageVariant,
    dpr: ImageDpr,
) -> String {
    let cloud = cloud_name
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .unwrap_or(DEFAULT_FALLBACK_CLOUD_NAME);
    let transform = variant.transformation_name(dpr);
    let trimmed_id = public_id.trim().trim_start_matches('/');

    format!("https://res.cloudinary.com/{cloud}/image/private/{transform}/{trimmed_id}")
}

/// Builds an `img` `srcset` attribute value with 1x and 2x resolution descriptors.
#[must_use]
pub fn build_srcset(cloud_name: Option<&str>, public_id: &str, variant: ImageVariant) -> String {
    let url_1x = build_image_url(cloud_name, public_id, variant, ImageDpr::Dpr1x);
    let url_2x = build_image_url(cloud_name, public_id, variant, ImageDpr::Dpr2x);

    format!("{url_1x} 1x, {url_2x} 2x")
}

/// Convenience helper to generate the 1x URL for an avatar image.
#[must_use]
pub fn avatar_url(cloud_name: Option<&str>, public_id: &str) -> String {
    build_image_url(cloud_name, public_id, ImageVariant::Avatar, ImageDpr::Dpr1x)
}

/// Convenience helper to generate the 1x/2x srcset for an avatar image.
#[must_use]
pub fn avatar_srcset(cloud_name: Option<&str>, public_id: &str) -> String {
    build_srcset(cloud_name, public_id, ImageVariant::Avatar)
}

/// Convenience helper to generate the 1x URL for a thumbnail image.
#[must_use]
pub fn thumb_url(cloud_name: Option<&str>, public_id: &str) -> String {
    build_image_url(cloud_name, public_id, ImageVariant::Thumb, ImageDpr::Dpr1x)
}

/// Convenience helper to generate the 1x/2x srcset for a thumbnail image.
#[must_use]
pub fn thumb_srcset(cloud_name: Option<&str>, public_id: &str) -> String {
    build_srcset(cloud_name, public_id, ImageVariant::Thumb)
}

/// Convenience helper to generate the 1x URL for a card cover image.
#[must_use]
pub fn card_url(cloud_name: Option<&str>, public_id: &str) -> String {
    build_image_url(cloud_name, public_id, ImageVariant::Card, ImageDpr::Dpr1x)
}

/// Convenience helper to generate the 1x/2x srcset for a card cover image.
#[must_use]
pub fn card_srcset(cloud_name: Option<&str>, public_id: &str) -> String {
    build_srcset(cloud_name, public_id, ImageVariant::Card)
}

/// Builds the 1x square discovery-card image URL.
#[must_use]
pub fn card_sq_url(cloud_name: Option<&str>, public_id: &str) -> String {
    build_image_url(
        cloud_name,
        public_id,
        ImageVariant::CardSquare,
        ImageDpr::Dpr1x,
    )
}

/// Builds the 1x and 2x square discovery-card image srcset.
#[must_use]
pub fn card_sq_srcset(cloud_name: Option<&str>, public_id: &str) -> String {
    build_srcset(cloud_name, public_id, ImageVariant::CardSquare)
}

/// Convenience helper to generate the 1x URL for a detail image.
#[must_use]
pub fn detail_url(cloud_name: Option<&str>, public_id: &str) -> String {
    build_image_url(cloud_name, public_id, ImageVariant::Detail, ImageDpr::Dpr1x)
}

/// Convenience helper to generate the 1x/2x srcset for a detail image.
#[must_use]
pub fn detail_srcset(cloud_name: Option<&str>, public_id: &str) -> String {
    build_srcset(cloud_name, public_id, ImageVariant::Detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformation_names_and_dimensions() {
        assert_eq!(
            ImageVariant::Avatar.transformation_name(ImageDpr::Dpr1x),
            "t_avatar"
        );
        assert_eq!(
            ImageVariant::Avatar.transformation_name(ImageDpr::Dpr2x),
            "t_avatar_2x"
        );
        assert_eq!(ImageVariant::Avatar.dimensions(ImageDpr::Dpr1x), (96, 96));
        assert_eq!(ImageVariant::Avatar.dimensions(ImageDpr::Dpr2x), (192, 192));

        assert_eq!(
            ImageVariant::Thumb.transformation_name(ImageDpr::Dpr1x),
            "t_thumb"
        );
        assert_eq!(
            ImageVariant::Thumb.transformation_name(ImageDpr::Dpr2x),
            "t_thumb_2x"
        );
        assert_eq!(ImageVariant::Thumb.dimensions(ImageDpr::Dpr1x), (120, 90));
        assert_eq!(ImageVariant::Thumb.dimensions(ImageDpr::Dpr2x), (240, 180));

        assert_eq!(
            ImageVariant::Card.transformation_name(ImageDpr::Dpr1x),
            "t_card"
        );
        assert_eq!(
            ImageVariant::Card.transformation_name(ImageDpr::Dpr2x),
            "t_card_2x"
        );
        assert_eq!(ImageVariant::Card.dimensions(ImageDpr::Dpr1x), (320, 240));
        assert_eq!(ImageVariant::Card.dimensions(ImageDpr::Dpr2x), (640, 480));

        assert_eq!(
            ImageVariant::CardSquare.transformation_name(ImageDpr::Dpr1x),
            "t_card_sq"
        );
        assert_eq!(
            ImageVariant::CardSquare.transformation_name(ImageDpr::Dpr2x),
            "t_card_sq_2x"
        );
        assert_eq!(
            ImageVariant::CardSquare.dimensions(ImageDpr::Dpr1x),
            (320, 320)
        );
        assert_eq!(
            ImageVariant::CardSquare.dimensions(ImageDpr::Dpr2x),
            (640, 640)
        );

        assert_eq!(
            ImageVariant::Detail.transformation_name(ImageDpr::Dpr1x),
            "t_detail"
        );
        assert_eq!(
            ImageVariant::Detail.transformation_name(ImageDpr::Dpr2x),
            "t_detail_2x"
        );
        assert_eq!(ImageVariant::Detail.dimensions(ImageDpr::Dpr1x), (640, 480));
        assert_eq!(ImageVariant::Detail.dimensions(ImageDpr::Dpr2x), (960, 720));
    }

    #[test]
    fn test_build_image_url_with_explicit_cloud_name() {
        let url = build_image_url(
            Some("my-cloud"),
            "users/123/abc.webp",
            ImageVariant::Avatar,
            ImageDpr::Dpr1x,
        );
        assert_eq!(
            url,
            "https://res.cloudinary.com/my-cloud/image/private/t_avatar/users/123/abc.webp"
        );
    }

    #[test]
    fn test_build_image_url_with_fallback_cloud_name() {
        let url = build_image_url(
            None,
            "offers/456/cover.jpg",
            ImageVariant::Card,
            ImageDpr::Dpr2x,
        );
        assert_eq!(
            url,
            "https://res.cloudinary.com/haven/image/private/t_card_2x/offers/456/cover.jpg"
        );
    }

    #[test]
    fn test_build_srcset_formats_correctly() {
        let srcset = avatar_srcset(Some("haven-prod"), "users/789/avatar");
        assert_eq!(
            srcset,
            "https://res.cloudinary.com/haven-prod/image/private/t_avatar/users/789/avatar 1x, https://res.cloudinary.com/haven-prod/image/private/t_avatar_2x/users/789/avatar 2x"
        );
    }
}
