use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::storage::BlobStorage;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Local filesystem implementation of `BlobStorage`.
/// Serves files locally and can be easily swapped for S3/GCS in cloud/distributed environments.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LocalStorage {
    base_dir: PathBuf,
    base_url: String,
}

#[allow(dead_code)]
impl LocalStorage {
    pub fn new(base_dir: impl Into<PathBuf>, base_url: impl Into<String>) -> Self {
        let base_dir = base_dir.into();
        let base_url = base_url.into();
        Self { base_dir, base_url }
    }

    /// Resolve a key to a sanitized filesystem path, preventing directory traversal.
    fn resolve_path(&self, key: &str) -> Result<PathBuf, DomainError> {
        let clean_key = key.trim_start_matches('/').trim_start_matches('\\');
        if clean_key.contains("..") {
            return Err(DomainError::new(
                ErrorCode::ErrorBadRequest,
                ErrorCode::ErrorBadRequest.as_str(),
            ));
        }
        Ok(self.base_dir.join(clean_key))
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new("./uploads", "/uploads")
    }
}

#[async_trait]
impl BlobStorage for LocalStorage {
    async fn upload(
        &self,
        key: &str,
        _content_type: &str,
        data: &[u8],
    ) -> Result<String, DomainError> {
        let path = self.resolve_path(key)?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!(target: "serve::storage", error = %e, path = %parent.display(), "Failed to create storage directory");
                DomainError::new(ErrorCode::StorageUploadFailed, ErrorCode::StorageUploadFailed.as_str())
            })?;
        }

        tokio::fs::write(&path, data).await.map_err(|e| {
            error!(target: "serve::storage", error = %e, path = %path.display(), "Failed to write file to storage");
            DomainError::new(ErrorCode::StorageUploadFailed, ErrorCode::StorageUploadFailed.as_str())
        })?;

        info!(target: "serve::storage", key = %key, bytes = data.len(), "Blob uploaded successfully");
        Ok(self.get_url(key))
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, DomainError> {
        let path = self.resolve_path(key)?;
        tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DomainError::new(ErrorCode::StorageFileNotFound, ErrorCode::StorageFileNotFound.as_str())
            } else {
                error!(target: "serve::storage", error = %e, path = %path.display(), "Failed to read file from storage");
                DomainError::new(ErrorCode::ErrorInternal, ErrorCode::ErrorInternal.as_str())
            }
        })
    }

    async fn delete(&self, key: &str) -> Result<bool, DomainError> {
        let path = self.resolve_path(key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {
                info!(target: "serve::storage", key = %key, "Blob deleted successfully");
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => {
                error!(target: "serve::storage", error = %e, path = %path.display(), "Failed to delete file from storage");
                Err(DomainError::new(
                    ErrorCode::StorageDeleteFailed,
                    ErrorCode::StorageDeleteFailed.as_str(),
                ))
            }
        }
    }

    async fn exists(&self, key: &str) -> bool {
        match self.resolve_path(key) {
            Ok(path) => Path::new(&path).exists(),
            Err(_) => false,
        }
    }

    fn get_url(&self, key: &str) -> String {
        let clean_key = key.trim_start_matches('/');
        format!("{}/{}", self.base_url.trim_end_matches('/'), clean_key)
    }
}
