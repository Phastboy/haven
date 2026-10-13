//! Cloudinary image storage adapter with signed upload and deletion support.

use haven_domain::ports::{ImageStorage, StorageError, StoredImage, async_trait};
use serde::Deserialize;
use sha1::Digest;
use std::env;

/// Configuration credentials for Cloudinary API interactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudinaryConfig {
    /// Cloud name account identifier.
    pub cloud_name: String,
    /// Public API key.
    pub api_key: String,
    /// Secret API key used for generating signatures.
    pub api_secret: String,
    /// Optional upload preset configuring incoming transformations in Cloudinary.
    pub upload_preset: Option<String>,
}

impl CloudinaryConfig {
    /// Loads Cloudinary credentials from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Unavailable`] if credentials are partially configured.
    pub fn from_env() -> Result<Option<Self>, StorageError> {
        let cloud_name = env::var("CLOUDINARY_CLOUD_NAME").ok();
        let api_key = env::var("CLOUDINARY_API_KEY").ok();
        let api_secret = env::var("CLOUDINARY_API_SECRET").ok();
        let upload_preset = env::var("CLOUDINARY_UPLOAD_PRESET").ok();

        let any_set = cloud_name.is_some() || api_key.is_some() || api_secret.is_some();

        match (cloud_name, api_key, api_secret) {
            (Some(name), Some(key), Some(secret))
                if !name.trim().is_empty() && !key.trim().is_empty() && !secret.trim().is_empty() =>
            {
                Ok(Some(Self {
                    cloud_name: name.trim().to_string(),
                    api_key: key.trim().to_string(),
                    api_secret: secret.trim().to_string(),
                    upload_preset: upload_preset.filter(|p| !p.trim().is_empty()),
                }))
            }
            _ if any_set => Err(StorageError::Unavailable(
                "Incomplete Cloudinary configuration: CLOUDINARY_CLOUD_NAME, CLOUDINARY_API_KEY, and CLOUDINARY_API_SECRET must all be set"
                    .to_string(),
            )),
            _ => Ok(None),
        }
    }
}

/// Computes the Cloudinary SHA-1 authentication signature for signed requests.
///
/// Parameters must be sorted alphabetically by key and concatenated as `key=value`
/// separated by `&`, followed directly by the `api_secret`.
#[must_use]
pub fn compute_cloudinary_signature(params: &[(&str, &str)], api_secret: &str) -> String {
    let mut sorted_params = params.to_vec();
    sorted_params.sort_by(|a, b| a.0.cmp(b.0));

    let mut to_sign = String::new();
    for (i, (k, v)) in sorted_params.iter().enumerate() {
        if i > 0 {
            to_sign.push('&');
        }
        to_sign.push_str(k);
        to_sign.push('=');
        to_sign.push_str(v);
    }
    to_sign.push_str(api_secret);

    let mut hasher = sha1::Sha1::new();
    hasher.update(to_sign.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Deserialize)]
struct CloudinaryUploadSuccess {
    public_id: String,
    width: u32,
    height: u32,
    format: String,
    bytes: usize,
}

#[derive(Debug, Deserialize)]
struct CloudinaryApiErrorResponse {
    error: CloudinaryApiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct CloudinaryApiErrorDetail {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CloudinaryDestroyResponse {
    result: String,
}

/// Cloudinary implementation of [`ImageStorage`].
#[derive(Debug, Clone)]
pub struct CloudinaryStorage {
    config: CloudinaryConfig,
    http_client: reqwest::Client,
}

impl CloudinaryStorage {
    /// Creates a new `CloudinaryStorage` instance with credentials and HTTP client.
    #[must_use]
    pub fn new(config: CloudinaryConfig, http_client: reqwest::Client) -> Self {
        Self {
            config,
            http_client,
        }
    }
}

#[async_trait]
impl ImageStorage for CloudinaryStorage {
    async fn upload(
        &self,
        folder: &str,
        public_id_suffix: &str,
        bytes: &[u8],
    ) -> Result<StoredImage, StorageError> {
        let timestamp_str = chrono::Utc::now().timestamp().to_string();

        let mut sign_params = vec![
            ("folder", folder),
            ("public_id", public_id_suffix),
            ("timestamp", &timestamp_str),
            ("type", "private"),
        ];

        if let Some(ref preset) = self.config.upload_preset {
            sign_params.push(("upload_preset", preset.as_str()));
        }

        let signature = compute_cloudinary_signature(&sign_params, &self.config.api_secret);

        let url = format!(
            "https://api.cloudinary.com/v1_1/{}/image/upload",
            self.config.cloud_name
        );

        let mut form = reqwest::multipart::Form::new()
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp_str)
            .text("folder", folder.to_string())
            .text("public_id", public_id_suffix.to_string())
            .text("type", "private")
            .text("signature", signature);

        if let Some(ref preset) = self.config.upload_preset {
            form = form.text("upload_preset", preset.clone());
        }

        let part = reqwest::multipart::Part::bytes(bytes.to_vec())
            .file_name(format!("{public_id_suffix}.jpg"))
            .mime_str("application/octet-stream")
            .map_err(|e| StorageError::UploadFailed(e.to_string()))?;

        form = form.part("file", part);

        let response = self
            .http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        if !status.is_success() {
            if let Ok(err_payload) =
                serde_json::from_slice::<CloudinaryApiErrorResponse>(&body_bytes)
            {
                return Err(StorageError::UploadFailed(err_payload.error.message));
            }
            let fallback_msg = String::from_utf8_lossy(&body_bytes);
            return Err(StorageError::UploadFailed(format!(
                "HTTP {status}: {fallback_msg}"
            )));
        }

        let success_payload = serde_json::from_slice::<CloudinaryUploadSuccess>(&body_bytes)
            .map_err(|e| {
                StorageError::UploadFailed(format!("failed to parse Cloudinary response: {e}"))
            })?;

        Ok(StoredImage {
            public_id: success_payload.public_id,
            width: success_payload.width,
            height: success_payload.height,
            format: success_payload.format,
            bytes: success_payload.bytes,
        })
    }

    async fn delete(&self, public_id: &str) -> Result<(), StorageError> {
        let timestamp_str = chrono::Utc::now().timestamp().to_string();

        let sign_params = [
            ("public_id", public_id),
            ("timestamp", &timestamp_str),
            ("type", "private"),
        ];

        let signature = compute_cloudinary_signature(&sign_params, &self.config.api_secret);

        let url = format!(
            "https://api.cloudinary.com/v1_1/{}/image/destroy",
            self.config.cloud_name
        );

        let form = reqwest::multipart::Form::new()
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp_str)
            .text("public_id", public_id.to_string())
            .text("type", "private")
            .text("signature", signature);

        let response = self
            .http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        if !status.is_success() {
            if let Ok(err_payload) =
                serde_json::from_slice::<CloudinaryApiErrorResponse>(&body_bytes)
            {
                return Err(StorageError::DeleteFailed(err_payload.error.message));
            }
            let fallback_msg = String::from_utf8_lossy(&body_bytes);
            return Err(StorageError::DeleteFailed(format!(
                "HTTP {status}: {fallback_msg}"
            )));
        }

        if let Ok(destroy_payload) =
            serde_json::from_slice::<CloudinaryDestroyResponse>(&body_bytes)
        {
            // "ok" means deleted; "not found" means it was already removed (idempotent delete)
            if destroy_payload.result == "ok" || destroy_payload.result == "not found" {
                return Ok(());
            }
            return Err(StorageError::DeleteFailed(destroy_payload.result));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cloudinary_signature() {
        let params = [
            ("folder", "offers/123"),
            ("public_id", "sample"),
            ("timestamp", "1315060510"),
        ];
        let secret = "abcd";

        // Expected concatenation: "folder=offers/123&public_id=sample&timestamp=1315060510abcd"
        let expected_sha1 = {
            let mut hasher = sha1::Sha1::new();
            hasher.update(b"folder=offers/123&public_id=sample&timestamp=1315060510abcd");
            hex::encode(hasher.finalize())
        };

        let calculated = compute_cloudinary_signature(&params, secret);
        assert_eq!(calculated, expected_sha1);
    }
}
