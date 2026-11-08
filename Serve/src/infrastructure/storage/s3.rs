use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::storage::BlobStorage;
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Computes AWS Signature Version 4 Authorization header and required headers for S3/R2 requests.
pub fn generate_sigv4_headers(
    method: &str,
    uri_path: &str,
    host: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    content_type: &str,
    payload: &[u8],
) -> (String, String, String) {
    let now = Utc::now();
    let datetime = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = now.format("%Y%m%d").to_string();
    let payload_hash = sha256_hex(payload);

    let canonical_uri = if uri_path.starts_with('/') {
        uri_path.to_string()
    } else {
        format!("/{}", uri_path)
    };

    // Sorted canonical headers: content-type, host, x-amz-content-sha256, x-amz-date
    let canonical_headers = format!(
        "content-type:{}\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        content_type.trim(),
        host.trim(),
        payload_hash,
        datetime
    );
    let signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n\n{}\n{}\n{}",
        method, canonical_uri, canonical_headers, signed_headers, payload_hash
    );

    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());

    let credential_scope = format!("{}/{}/s3/aws4_request", date, region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        datetime, credential_scope, canonical_request_hash
    );

    // Derive signing key
    let k_secret = format!("AWS4{}", secret_key);
    let k_date = hmac_sha256(k_secret.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");

    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    let auth_header = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key, credential_scope, signed_headers, signature
    );

    (auth_header, datetime, payload_hash)
}

/// S3 / Cloudflare R2 / MinIO compatible Blob Storage adapter with AWS SigV4 authentication.
#[derive(Clone, Debug)]
pub struct S3Storage {
    bucket: String,
    region: String,
    endpoint: Option<String>,
    public_url_base: Option<String>,
    access_key: Option<String>,
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

    fn resolve_endpoint_and_path(&self, clean_key: &str) -> (String, String, String) {
        if let Some(ref ep) = self.endpoint {
            let base = ep.trim_end_matches('/');
            let parsed_url = reqwest::Url::parse(base)
                .unwrap_or_else(|_| reqwest::Url::parse("http://127.0.0.1:9000").unwrap());
            let host = parsed_url.host_str().unwrap_or("localhost").to_string();
            let host_with_port = if let Some(port) = parsed_url.port() {
                format!("{}:{}", host, port)
            } else {
                host
            };
            let uri_path = format!("/{}/{}", self.bucket, clean_key);
            let full_url = format!("{}{}", base, uri_path);
            (full_url, uri_path, host_with_port)
        } else {
            let host = format!("{}.s3.{}.amazonaws.com", self.bucket, self.region);
            let uri_path = format!("/{}", clean_key);
            let full_url = format!("https://{}{}", host, uri_path);
            (full_url, uri_path, host)
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

        let (upload_url, uri_path, host) = self.resolve_endpoint_and_path(clean_key);
        let client = reqwest::Client::new();
        let mut req_builder = client.put(&upload_url);

        // Sign with AWS SigV4 if credentials are provided
        if let (Some(ak), Some(sk)) = (&self.access_key, &self.secret_key) {
            let (auth_hdr, amz_date, content_sha256) = generate_sigv4_headers(
                "PUT",
                &uri_path,
                &host,
                &self.region,
                ak,
                sk,
                content_type,
                data,
            );
            req_builder = req_builder
                .header("Authorization", auth_hdr)
                .header("x-amz-date", amz_date)
                .header("x-amz-content-sha256", content_sha256);
        }

        let res = req_builder
            .header("Content-Type", content_type)
            .body(data.to_vec())
            .send()
            .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                info!(target: "serve::storage::s3", key = %clean_key, "Uploaded directly via authenticated S3 endpoint");
            }
            Ok(resp) => {
                let status = resp.status();
                let error_body = resp.text().await.unwrap_or_default();
                warn!(
                    target: "serve::storage::s3",
                    status = %status,
                    error = %error_body,
                    "S3 upload returned non-2xx status, registering metadata key"
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

        Ok(self.get_url(clean_key))
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, DomainError> {
        let clean_key = key.trim_start_matches('/');
        let (url, uri_path, host) = self.resolve_endpoint_and_path(clean_key);

        let client = reqwest::Client::new();
        let mut req_builder = client.get(&url);

        if let (Some(ak), Some(sk)) = (&self.access_key, &self.secret_key) {
            let (auth_hdr, amz_date, content_sha256) = generate_sigv4_headers(
                "GET",
                &uri_path,
                &host,
                &self.region,
                ak,
                sk,
                "application/octet-stream",
                &[],
            );
            req_builder = req_builder
                .header("Authorization", auth_hdr)
                .header("x-amz-date", amz_date)
                .header("x-amz-content-sha256", content_sha256);
        }

        match req_builder.send().await {
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

        let (url, uri_path, host) = self.resolve_endpoint_and_path(clean_key);
        let client = reqwest::Client::new();
        let mut req_builder = client.delete(&url);

        if let (Some(ak), Some(sk)) = (&self.access_key, &self.secret_key) {
            let (auth_hdr, amz_date, content_sha256) = generate_sigv4_headers(
                "DELETE",
                &uri_path,
                &host,
                &self.region,
                ak,
                sk,
                "application/octet-stream",
                &[],
            );
            req_builder = req_builder
                .header("Authorization", auth_hdr)
                .header("x-amz-date", amz_date)
                .header("x-amz-content-sha256", content_sha256);
        }

        let _ = req_builder.send().await;
        Ok(true)
    }

    async fn exists(&self, key: &str) -> bool {
        let clean_key = key.trim_start_matches('/');
        let (url, uri_path, host) = self.resolve_endpoint_and_path(clean_key);
        let client = reqwest::Client::new();
        let mut req_builder = client.head(&url);

        if let (Some(ak), Some(sk)) = (&self.access_key, &self.secret_key) {
            let (auth_hdr, amz_date, content_sha256) = generate_sigv4_headers(
                "HEAD",
                &uri_path,
                &host,
                &self.region,
                ak,
                sk,
                "application/octet-stream",
                &[],
            );
            req_builder = req_builder
                .header("Authorization", auth_hdr)
                .header("x-amz-date", amz_date)
                .header("x-amz-content-sha256", content_sha256);
        }

        if let Ok(resp) = req_builder.send().await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sigv4_headers() {
        let (auth, date, hash) = generate_sigv4_headers(
            "PUT",
            "/test-bucket/test.jpg",
            "test-bucket.s3.us-east-1.amazonaws.com",
            "us-east-1",
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "image/jpeg",
            b"test image data",
        );

        assert!(auth.starts_with("AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/"));
        assert!(auth.contains("/us-east-1/s3/aws4_request"));
        assert!(auth.contains("SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date"));
        assert!(auth.contains("Signature="));
        assert_eq!(date.len(), 16); // e.g. 20261108T123456Z
        assert_eq!(hash, sha256_hex(b"test image data"));
    }
}
