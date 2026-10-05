use axum::{
    extract::{Extension, Multipart},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::storage::BlobStorage;
use crate::infrastructure::auth::verify_jwt;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::storage::StorageService;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB

/// Inspect raw file bytes to determine and validate authentic image format via Magic Bytes.
/// Returns `Some((extension, canonical_mime_type))` if valid image format, or `None` if invalid.
pub fn validate_image_magic_bytes(data: &[u8]) -> Option<(&'static str, &'static str)> {
    if data.is_empty() {
        return None;
    }

    // JPEG / JPG: FF D8 FF
    if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some(("jpg", "image/jpeg"));
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.len() >= 8 && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(("png", "image/png"));
    }

    // GIF: GIF87a or GIF89a
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return Some(("gif", "image/gif"));
    }

    // WEBP: RIFF....WEBP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some(("webp", "image/webp"));
    }

    // HEIF / HEIC / AVIF: ISOBMFF container starting with 'ftyp' box type at offset 4
    if data.len() >= 12 && &data[4..8] == b"ftyp" {
        let inspect_len = data.len().min(64);
        let header = &data[8..inspect_len];

        // AVIF: AV1 Image File Format
        if header.windows(4).any(|w| w == b"avif" || w == b"avis") {
            return Some(("avif", "image/avif"));
        }

        // HEIC: High Efficiency Image Coding (HEVC-based)
        if header.windows(4).any(|w| {
            matches!(w, b"heic" | b"heix" | b"heim" | b"heis" | b"hevc" | b"hevx")
        }) {
            return Some(("heic", "image/heic"));
        }

        // Generic HEIF / MIAF
        if header.windows(4).any(|w| matches!(w, b"mif1" | b"msf1" | b"miaf")) {
            return Some(("heif", "image/heif"));
        }
    }

    None
}

/// POST /api/upload
/// Accepts multipart/form-data with an image file (`file` or `image` field).
/// Requires Bearer JWT token in Authorization header for authenticated access.
pub async fn upload_file_handler(
    Extension(storage): Extension<StorageService>,
    Extension(config): Extension<AppConfig>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 1. Mandatory JWT Authentication
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(h) => h.trim(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "UNAUTHORIZED",
                    "message": "Authorization header with Bearer token is required for file uploads"
                })),
            );
        }
    };

    let token = match auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
    {
        Some(t) => t.trim(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "UNAUTHORIZED",
                    "message": "Malformed authorization token. Expected 'Bearer <token>'"
                })),
            );
        }
    };

    let auth_user = match verify_jwt(token, &config.auth.jwt_secret) {
        Ok(u) => u,
        Err(err) => {
            warn!(target: "serve::upload", error = %err, "Upload rejected due to invalid authorization token");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "UNAUTHORIZED",
                    "message": "Invalid or expired authorization token"
                })),
            );
        }
    };

    // 2. Parse Multipart payload
    let mut uploaded_file = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "image" {
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    error!(target: "serve::upload", error = %err, "Failed to read upload payload");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "READ_ERROR",
                            "message": "Failed to read uploaded file data"
                        })),
                    );
                }
            };

            if data.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "EMPTY_FILE",
                        "message": "Uploaded file is empty"
                    })),
                );
            }

            if data.len() > MAX_FILE_SIZE {
                return (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    Json(json!({
                        "error": "FILE_TOO_LARGE",
                        "message": format!("File size exceeds limit of {} MB", MAX_FILE_SIZE / (1024 * 1024))
                    })),
                );
            }

            // 3. Validate Authentic File Content via Magic Bytes
            let (ext, validated_content_type) = match validate_image_magic_bytes(&data) {
                Some((ext, mime)) => (ext, mime),
                None => {
                    warn!(
                        target: "serve::upload",
                        user_id = %auth_user.user_id,
                        bytes = data.len(),
                        "Upload rejected: Magic byte signature check failed"
                    );
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "INVALID_FILE_SIGNATURE",
                            "message": "Uploaded file content does not match allowed image signatures (JPEG, PNG, GIF, WEBP, HEIC, HEIF, AVIF)"
                        })),
                    );
                }
            };

            let unique_key = format!("{}.{}", Uuid::new_v4(), ext);

            match storage
                .upload(&unique_key, validated_content_type, &data)
                .await
            {
                Ok(url) => {
                    info!(
                        target: "serve::upload",
                        key = %unique_key,
                        size = data.len(),
                        user_id = %auth_user.user_id,
                        username = %auth_user.username,
                        content_type = %validated_content_type,
                        "Image uploaded successfully"
                    );
                    uploaded_file = Some(json!({
                        "url": url,
                        "key": unique_key,
                        "size": data.len(),
                        "content_type": validated_content_type
                    }));
                    break;
                }
                Err(err) => {
                    error!(target: "serve::upload", error = %err, "Failed to store image in blob storage");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "STORAGE_ERROR",
                            "message": "Failed to persist uploaded file"
                        })),
                    );
                }
            }
        }
    }

    if let Some(res) = uploaded_file {
        (StatusCode::CREATED, Json(res))
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "MISSING_FILE",
                "message": "No valid file or image field was found in multipart form-data"
            })),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_image_magic_bytes_jpeg() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(validate_image_magic_bytes(&jpeg), Some(("jpg", "image/jpeg")));
    }

    #[test]
    fn test_validate_image_magic_bytes_png() {
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(validate_image_magic_bytes(&png), Some(("png", "image/png")));
    }

    #[test]
    fn test_validate_image_magic_bytes_gif() {
        let gif87 = b"GIF87a...";
        let gif89 = b"GIF89a...";
        assert_eq!(validate_image_magic_bytes(gif87), Some(("gif", "image/gif")));
        assert_eq!(validate_image_magic_bytes(gif89), Some(("gif", "image/gif")));
    }

    #[test]
    fn test_validate_image_magic_bytes_webp() {
        let webp = b"RIFF\x00\x00\x00\x00WEBPVP8 ...";
        assert_eq!(validate_image_magic_bytes(webp), Some(("webp", "image/webp")));
    }

    #[test]
    fn test_validate_image_magic_bytes_heic() {
        let mut heic = vec![0x00, 0x00, 0x00, 0x18];
        heic.extend_from_slice(b"ftypheic");
        heic.extend_from_slice(b"\x00\x00\x00\x00mif1heic");
        assert_eq!(validate_image_magic_bytes(&heic), Some(("heic", "image/heic")));
    }

    #[test]
    fn test_validate_image_magic_bytes_heif() {
        let mut heif = vec![0x00, 0x00, 0x00, 0x18];
        heif.extend_from_slice(b"ftypmif1");
        heif.extend_from_slice(b"\x00\x00\x00\x00mif1miaf");
        assert_eq!(validate_image_magic_bytes(&heif), Some(("heif", "image/heif")));
    }

    #[test]
    fn test_validate_image_magic_bytes_avif() {
        let mut avif = vec![0x00, 0x00, 0x00, 0x1c];
        avif.extend_from_slice(b"ftypavif");
        avif.extend_from_slice(b"\x00\x00\x00\x00mif1miafavif");
        assert_eq!(validate_image_magic_bytes(&avif), Some(("avif", "image/avif")));
    }

    #[test]
    fn test_validate_image_magic_bytes_invalid() {
        assert_eq!(validate_image_magic_bytes(&[]), None);
        assert_eq!(validate_image_magic_bytes(b"HELLO WORLD"), None);
        assert_eq!(validate_image_magic_bytes(b"\x00\x00\x00\x18ftypmp42"), None);
    }
}

