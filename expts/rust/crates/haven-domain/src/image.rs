//! Image value objects, identifiers, format detection, and validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::error::ImageValidationError;
use crate::offer::OfferId;

/// Maximum size allowed for an image upload in bytes (5 MB).
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

/// Minimum image position index (1-based).
pub const MIN_IMAGE_POSITION: u8 = 1;

/// Maximum image position index (1-based).
pub const MAX_IMAGE_POSITION: u8 = 5;

/// Maximum character length for a public image identifier.
pub const MAX_PUBLIC_ID_CHARS: usize = 120;

/// Newtype wrapping a UUID that identifies an uploaded image record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageId(Uuid);

impl ImageId {
    /// Generates a new random `ImageId`.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Constructs an `ImageId` from an existing UUID.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying raw UUID.
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for ImageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl fmt::Display for ImageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Strongly typed position index for offer images (constrained between 1 and 5 inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ImagePosition(u8);

impl ImagePosition {
    /// Creates a new `ImagePosition` if within valid bounds (1..=5).
    ///
    /// # Errors
    ///
    /// Returns [`ImageValidationError::InvalidPosition`] if out of bounds.
    pub fn new(pos: u8) -> Result<Self, ImageValidationError> {
        if (MIN_IMAGE_POSITION..=MAX_IMAGE_POSITION).contains(&pos) {
            Ok(Self(pos))
        } else {
            Err(ImageValidationError::InvalidPosition(pos))
        }
    }

    /// Returns the raw position number (1..=5).
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Returns true if this position is the cover image (position 1).
    #[must_use]
    pub fn is_cover(&self) -> bool {
        self.0 == 1
    }
}

impl TryFrom<u8> for ImagePosition {
    type Error = ImageValidationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for ImagePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Width and height dimensions in positive pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    width: u32,
    height: u32,
}

impl ImageDimensions {
    /// Constructs dimensions from positive width and height.
    ///
    /// # Errors
    ///
    /// Returns [`ImageValidationError::InvalidDimensions`] if either dimension is 0.
    pub fn new(width: u32, height: u32) -> Result<Self, ImageValidationError> {
        if width == 0 || height == 0 {
            Err(ImageValidationError::InvalidDimensions)
        } else {
            Ok(Self { width, height })
        }
    }

    /// Returns the width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Default for ImageDimensions {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 960,
        }
    }
}

/// Validated opaque identifier for stored assets (e.g. Cloudinary public ID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImagePublicId(String);

impl ImagePublicId {
    /// Validates and parses an image public ID.
    ///
    /// # Errors
    ///
    /// Returns [`ImageValidationError::InvalidPublicId`] if blank, too long, or containing disallowed characters.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, ImageValidationError> {
        let trimmed = raw.as_ref().trim();
        if trimmed.is_empty() {
            return Err(ImageValidationError::InvalidPublicId(
                "public_id cannot be blank".to_string(),
            ));
        }
        if trimmed.chars().count() > MAX_PUBLIC_ID_CHARS {
            return Err(ImageValidationError::InvalidPublicId(
                "public_id exceeds maximum length of 120".to_string(),
            ));
        }
        let is_valid = trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.');
        if !is_valid {
            return Err(ImageValidationError::InvalidPublicId(
                "public_id contains invalid characters".to_string(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the public ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ImagePublicId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Supported raster image formats for upload and delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    /// Joint Photographic Experts Group image (`image/jpeg`).
    Jpeg,
    /// Portable Network Graphics image (`image/png`).
    Png,
    /// WebP image (`image/webp`).
    Webp,
}

impl ImageFormat {
    /// Returns the canonical MIME type string for this format.
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
        }
    }

    /// Returns the canonical file extension without leading period.
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }
}

/// Inspects the header magic bytes of a file slice to identify and validate supported formats.
///
/// # Errors
///
/// Returns [`ImageValidationError::EmptyFile`] if the slice is empty,
/// [`ImageValidationError::FileTooLarge`] if the slice exceeds [`MAX_IMAGE_BYTES`],
/// or [`ImageValidationError::UnsupportedFormat`] if the magic bytes do not match JPEG, PNG, or WebP.
pub fn validate_image_magic_bytes(bytes: &[u8]) -> Result<ImageFormat, ImageValidationError> {
    if bytes.is_empty() {
        return Err(ImageValidationError::EmptyFile);
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageValidationError::FileTooLarge);
    }

    // JPEG: begins with 0xFF, 0xD8, 0xFF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(ImageFormat::Jpeg);
    }

    // PNG: begins with 0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n'
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(ImageFormat::Png);
    }

    // WebP: RIFF header (bytes 0..4) and WEBP signature (bytes 8..12)
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return Ok(ImageFormat::Webp);
    }

    Err(ImageValidationError::UnsupportedFormat)
}

/// Offer image record linking an uploaded asset to an offer with ordering position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferImage {
    /// Unique identifier of the image record.
    pub id: ImageId,
    /// Associated offer identifier.
    pub offer_id: OfferId,
    /// Public CDN asset ID.
    pub public_id: ImagePublicId,
    /// Ordered position (1..=5).
    pub position: ImagePosition,
    /// Dimensions in pixels.
    pub dimensions: ImageDimensions,
    /// Timestamp of image upload.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_image_id_generation_and_conversion() {
        let id1 = ImageId::generate();
        let id2 = ImageId::generate();
        assert_ne!(id1, id2);

        let uuid = id1.as_uuid();
        assert_eq!(ImageId::from_uuid(uuid), id1);
        assert_eq!(ImageId::from(uuid), id1);
        assert_eq!(format!("{id1}"), uuid.to_string());
    }

    #[test]
    fn test_image_position_bounds() {
        assert_eq!(ImagePosition::new(1).unwrap().as_u8(), 1);
        assert!(ImagePosition::new(1).unwrap().is_cover());
        assert_eq!(ImagePosition::new(5).unwrap().as_u8(), 5);
        assert!(!ImagePosition::new(5).unwrap().is_cover());

        assert!(matches!(
            ImagePosition::new(0),
            Err(ImageValidationError::InvalidPosition(0))
        ));
        assert!(matches!(
            ImagePosition::new(6),
            Err(ImageValidationError::InvalidPosition(6))
        ));
    }

    #[test]
    fn test_image_dimensions() {
        let dims = ImageDimensions::new(1280, 720).unwrap();
        assert_eq!(dims.width(), 1280);
        assert_eq!(dims.height(), 720);

        assert_eq!(
            ImageDimensions::new(0, 100),
            Err(ImageValidationError::InvalidDimensions)
        );
        assert_eq!(
            ImageDimensions::new(100, 0),
            Err(ImageValidationError::InvalidDimensions)
        );
    }

    #[test]
    fn test_image_public_id_parsing() {
        assert!(ImagePublicId::parse("offers/123/sample_image").is_ok());
        assert!(ImagePublicId::parse("users/456/avatar-v1.webp").is_ok());

        assert!(matches!(
            ImagePublicId::parse("   "),
            Err(ImageValidationError::InvalidPublicId(_))
        ));
        assert!(matches!(
            ImagePublicId::parse("invalid public id with spaces"),
            Err(ImageValidationError::InvalidPublicId(_))
        ));
        assert!(matches!(
            ImagePublicId::parse("image<script>"),
            Err(ImageValidationError::InvalidPublicId(_))
        ));

        let long_id = "a".repeat(121);
        assert!(matches!(
            ImagePublicId::parse(long_id),
            Err(ImageValidationError::InvalidPublicId(_))
        ));
    }

    #[test]
    fn test_magic_byte_detection() {
        // JPEG
        let jpeg_bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(
            validate_image_magic_bytes(&jpeg_bytes),
            Ok(ImageFormat::Jpeg)
        );

        // PNG
        let png_bytes = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(validate_image_magic_bytes(&png_bytes), Ok(ImageFormat::Png));

        // WebP
        let mut webp_bytes = Vec::new();
        webp_bytes.extend_from_slice(b"RIFF");
        webp_bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        webp_bytes.extend_from_slice(b"WEBP");
        assert_eq!(
            validate_image_magic_bytes(&webp_bytes),
            Ok(ImageFormat::Webp)
        );

        // SVG / HTML / Text rejection (Strictly No SVG)
        assert_eq!(
            validate_image_magic_bytes(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"),
            Err(ImageValidationError::UnsupportedFormat)
        );
        assert_eq!(
            validate_image_magic_bytes(b"<?xml version=\"1.0\"?>"),
            Err(ImageValidationError::UnsupportedFormat)
        );
        assert_eq!(
            validate_image_magic_bytes(b"GIF89a"),
            Err(ImageValidationError::UnsupportedFormat)
        );

        // Empty file
        assert_eq!(
            validate_image_magic_bytes(&[]),
            Err(ImageValidationError::EmptyFile)
        );

        // Truncated magic bytes
        assert_eq!(
            validate_image_magic_bytes(&[0xFF, 0xD8]),
            Err(ImageValidationError::UnsupportedFormat)
        );
        assert_eq!(
            validate_image_magic_bytes(b"RIFF"),
            Err(ImageValidationError::UnsupportedFormat)
        );
        assert_eq!(
            validate_image_magic_bytes(b"RIFF1234AVIF"),
            Err(ImageValidationError::UnsupportedFormat)
        );

        // File too large boundary check
        let exact_max = vec![0xFF, 0xD8, 0xFF, 0x00];
        assert_eq!(validate_image_magic_bytes(&exact_max), Ok(ImageFormat::Jpeg));

        // Excess bytes beyond 5MB
        let oversized = vec![0; MAX_IMAGE_BYTES + 1];
        assert_eq!(
            validate_image_magic_bytes(&oversized),
            Err(ImageValidationError::FileTooLarge)
        );
    }

    #[test]
    fn test_image_format_helpers() {
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Webp.mime_type(), "image/webp");
        assert_eq!(ImageFormat::Webp.extension(), "webp");
    }
}
