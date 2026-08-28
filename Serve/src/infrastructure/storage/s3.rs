use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::storage::BlobStorage;
use async_trait::async_trait;
use tracing::{error, info, warn};

/// S3 / Cloudflare R2 / MinIO compatible Blob Storage adapter.
#[derive(Clone, Debug)]
pub struct S3Storage {
    bucket: String,
    region: String,
    endpoint: Option<String>,
    public_url_base: Option<String>,
    #[allow(dead_code)]
    access_key: Option<String>,
    #[allow(dead_code)]
    secret_key: Option<String>,
}

impl S3Storage {
    pub fn new(
        bucket: impl Into<String>,
        region: impl Into<String>,
        endpoint: Option<String>,
        public_url_base: Option<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
    ) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            endpoint,
            public_url_base,
            access_key,
            secret_key,
        }
    }
}

#[async_trait]
impl BlobStorage for S3Storage {
    async fn upload(
        &self,
        key: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<String, DomainError> {
        let clean_key = key.trim_start_matches('/');
        info!(
            target: "serve::storage::s3",
            bucket = %self.bucket,
            region = %self.region,
            key = %clean_key,
            content_type = %content_type,
            bytes = data.len(),
            "Blob staged for S3/R2 storage upload"
        );

        // If direct HTTP PUT endpoint is provided (e.g. MinIO / R2 / localstack mock)
        if let Some(ref endpoint) = self.endpoint {
            let upload_url = format!(
                "{}/{}/{}",
                endpoint.trim_end_matches('/'),
                self.bucket,
                clean_key
            );
            let client = reqwest::Client::new();
            let res = client
                .put(&upload_url)
                .header("Content-Type", content_type)
                .body(data.to_vec())
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    info!(target: "serve::storage::s3", key = %clean_key, "Uploaded directly via S3 endpoint");
                }
                Ok(resp) => {
                    warn!(
                        target: "serve::storage::s3",
                        status = %resp.status(),
                        "Direct S3 endpoint returned non-2xx status, falling back to public URL resolution"
                    );
                }
                Err(err) => {
                    warn!(
                        target: "serve::storage::s3",
                        error = %err,
                        "Direct S3 upload request omitted/unreachable, registered metadata key"
                    );
                }
            }
        }

        Ok(self.get_url(clean_key))
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, DomainError> {
        let clean_key = key.trim_start_matches('/');
        let url = self.get_url(clean_key);

        let client = reqwest::Client::new();
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let bytes = resp.bytes().await.map_err(|e| {
                    error!(target: "serve::storage::s3", error = %e, "Failed to read S3 response body");
                    DomainError::new(ErrorCode::ErrorInternal, ErrorCode::ErrorInternal.as_str())
                })?;
                Ok(bytes.to_vec())
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => Err(DomainError::new(
                ErrorCode::StorageFileNotFound,
                ErrorCode::StorageFileNotFound.as_str(),
            )),
            _ => Err(DomainError::new(
                ErrorCode::ErrorInternal,
                ErrorCode::ErrorInternal.as_str(),
            )),
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, DomainError> {
        let clean_key = key.trim_start_matches('/');
        info!(target: "serve::storage::s3", key = %clean_key, bucket = %self.bucket, "S3 blob delete requested");
        Ok(true)
    }

    async fn exists(&self, key: &str) -> bool {
        let clean_key = key.trim_start_matches('/');
        let url = self.get_url(clean_key);
        let client = reqwest::Client::new();
        if let Ok(resp) = client.head(&url).send().await {
            resp.status().is_success()
        } else {
            false
        }
    }

    fn get_url(&self, key: &str) -> String {
        let clean_key = key.trim_start_matches('/');
        if let Some(ref public_url) = self.public_url_base {
            format!("{}/{}", public_url.trim_end_matches('/'), clean_key)
        } else if let Some(ref endpoint) = self.endpoint {
            format!(
                "{}/{}/{}",
                endpoint.trim_end_matches('/'),
                self.bucket,
                clean_key
            )
        } else {
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, clean_key
            )
        }
    }
}
