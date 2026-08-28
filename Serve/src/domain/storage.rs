use crate::domain::errors::DomainError;
use async_trait::async_trait;

/// Blob storage abstraction trait.
/// Default implementation is local filesystem (`LocalStorage`).
/// Can be replaced with S3 / GCS / Azure Blob for distributed cloud storage without touching domain logic.
#[allow(dead_code)]
#[async_trait]
pub trait BlobStorage: Send + Sync {
    /// Upload file data and return its public URL or storage path
    async fn upload(
        &self,
        key: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<String, DomainError>;

    /// Read raw file bytes by key
    async fn read(&self, key: &str) -> Result<Vec<u8>, DomainError>;

    /// Delete file by key
    async fn delete(&self, key: &str) -> Result<bool, DomainError>;

    /// Check if file exists
    async fn exists(&self, key: &str) -> bool;

    /// Get public accessible URL for the blob
    fn get_url(&self, key: &str) -> String;
}
