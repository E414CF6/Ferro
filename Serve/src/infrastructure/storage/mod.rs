pub mod local;
pub mod s3;

pub use local::LocalStorage;
pub use s3::S3Storage;

use crate::domain::errors::DomainError;
use crate::domain::storage::BlobStorage;
use crate::infrastructure::config::{StorageConfig, StorageDriver};
use async_trait::async_trait;
use std::sync::Arc;

/// Unified Storage service abstraction supporting both Local filesystem and S3/R2 cloud storage.
#[derive(Clone, Debug)]
pub enum StorageService {
    Local(LocalStorage),
    S3(S3Storage),
}

impl StorageService {
    pub fn from_config(config: &StorageConfig) -> Self {
        match config.driver {
            StorageDriver::Local => Self::Local(LocalStorage::new(
                &config.local_path,
                &config.local_base_url,
            )),
            StorageDriver::S3 => Self::S3(S3Storage::new(
                &config.s3_bucket,
                &config.s3_region,
                config.s3_endpoint.clone(),
                config.s3_public_url.clone(),
                config.s3_access_key.clone(),
                config.s3_secret_key.clone(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn into_arc(self) -> Arc<dyn BlobStorage> {
        match self {
            StorageService::Local(s) => Arc::new(s),
            StorageService::S3(s) => Arc::new(s),
        }
    }
}

impl Default for StorageService {
    fn default() -> Self {
        Self::Local(LocalStorage::default())
    }
}

#[async_trait]
impl BlobStorage for StorageService {
    async fn upload(
        &self,
        key: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<String, DomainError> {
        match self {
            StorageService::Local(s) => s.upload(key, content_type, data).await,
            StorageService::S3(s) => s.upload(key, content_type, data).await,
        }
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, DomainError> {
        match self {
            StorageService::Local(s) => s.read(key).await,
            StorageService::S3(s) => s.read(key).await,
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, DomainError> {
        match self {
            StorageService::Local(s) => s.delete(key).await,
            StorageService::S3(s) => s.delete(key).await,
        }
    }

    async fn exists(&self, key: &str) -> bool {
        match self {
            StorageService::Local(s) => s.exists(key).await,
            StorageService::S3(s) => s.exists(key).await,
        }
    }

    fn get_url(&self, key: &str) -> String {
        match self {
            StorageService::Local(s) => s.get_url(key),
            StorageService::S3(s) => s.get_url(key),
        }
    }
}
