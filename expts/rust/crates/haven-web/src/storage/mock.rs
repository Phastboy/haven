//! In-memory mock implementation of [`haven_domain::ports::ImageStorage`] for testing and local dev.

use haven_domain::ports::{ImageStorage, StorageError, StoredImage, async_trait};
use std::sync::{Arc, Mutex};

/// In-memory image storage for testing environments and offline development.
#[derive(Debug, Clone, Default)]
pub struct MockImageStorage {
    uploaded: Arc<Mutex<Vec<StoredImage>>>,
    deleted: Arc<Mutex<Vec<String>>>,
    delete_error: Arc<Mutex<Option<String>>>,
}

impl MockImageStorage {
    /// Creates a new empty `MockImageStorage`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            uploaded: Arc::new(Mutex::new(Vec::new())),
            deleted: Arc::new(Mutex::new(Vec::new())),
            delete_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Configures the mock to fail deletion attempts with the specified error message.
    #[must_use]
    pub fn with_delete_error(self, err: impl Into<String>) -> Self {
        if let Ok(mut guard) = self.delete_error.lock() {
            *guard = Some(err.into());
        }
        self
    }

    /// Returns a snapshot of all successfully uploaded image records.
    #[must_use]
    pub fn uploaded_images(&self) -> Vec<StoredImage> {
        self.uploaded
            .lock()
            .map_or_else(|_| Vec::new(), |guard| guard.clone())
    }

    /// Returns a snapshot of all requested deletion public IDs.
    #[must_use]
    pub fn deleted_ids(&self) -> Vec<String> {
        self.deleted
            .lock()
            .map_or_else(|_| Vec::new(), |guard| guard.clone())
    }
}

#[async_trait]
impl ImageStorage for MockImageStorage {
    async fn upload(
        &self,
        folder: &str,
        public_id_suffix: &str,
        bytes: &[u8],
    ) -> Result<StoredImage, StorageError> {
        let public_id = format!("{folder}/{public_id_suffix}");
        let stored = StoredImage {
            public_id,
            width: 1280,
            height: 960,
            format: "webp".to_string(),
            bytes: bytes.len(),
        };

        if let Ok(mut guard) = self.uploaded.lock() {
            guard.push(stored.clone());
        }

        Ok(stored)
    }

    async fn delete(&self, public_id: &str) -> Result<(), StorageError> {
        if let Some(err) = self
            .delete_error
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
        {
            return Err(StorageError::DeleteFailed(err));
        }
        if let Ok(mut guard) = self.deleted.lock() {
            guard.push(public_id.to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_storage_upload_and_delete() {
        let storage = MockImageStorage::new();
        let res = storage
            .upload("offers/123", "cover", b"sample-bytes")
            .await
            .unwrap();

        assert_eq!(res.public_id, "offers/123/cover");
        assert_eq!(res.width, 1280);
        assert_eq!(res.height, 960);
        assert_eq!(res.bytes, 12);

        assert_eq!(storage.uploaded_images().len(), 1);

        storage.delete("offers/123/cover").await.unwrap();
        assert_eq!(storage.deleted_ids(), vec!["offers/123/cover"]);
    }
}
