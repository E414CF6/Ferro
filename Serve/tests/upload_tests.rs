use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serve::graphql::build_schema;
use serve::infrastructure::{
    auth::create_jwt, config::AppConfig, db::postgres::Database, pubsub::MessageBroker,
};
use serve::routes::create_router;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_upload_and_static_file_serving() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

    let db = match Database::connect_url(&database_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test: PostgreSQL connection failed: {}", e);
            return;
        }
    };

    let config = AppConfig::default();
    let schema = build_schema(db.clone(), config.auth.clone(), MessageBroker::default());
    let app = create_router(schema, db, config.clone());

    let user_id = Uuid::new_v4();
    let token = create_jwt(user_id, "test_uploader", &config.auth.jwt_secret, 1).unwrap();

    let boundary = "------------------------boundary12345";
    let header_part = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"test_avatar.png\"\r\n\
        Content-Type: image/png\r\n\r\n"
    );
    let footer_part = format!("\r\n--{boundary}--\r\n");

    let png_bytes = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0x63, 0x34,
    ];

    let mut body_vec = Vec::new();
    body_vec.extend_from_slice(header_part.as_bytes());
    body_vec.extend_from_slice(&png_bytes);
    body_vec.extend_from_slice(footer_part.as_bytes());

    // 1. Unauthenticated upload -> Expect 401 UNAUTHORIZED
    let unauth_req = Request::builder()
        .method("POST")
        .uri("/api/upload")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body_vec.clone()))
        .unwrap();

    let unauth_res = app.clone().oneshot(unauth_req).await.unwrap();
    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    // 2. Upload with invalid magic bytes (fake image) -> Expect 400 BAD_REQUEST
    let fake_image_bytes = b"NOT_A_REAL_IMAGE_FILE_CONTENT";
    let mut fake_body_vec = Vec::new();
    fake_body_vec.extend_from_slice(header_part.as_bytes());
    fake_body_vec.extend_from_slice(fake_image_bytes);
    fake_body_vec.extend_from_slice(footer_part.as_bytes());

    let fake_req = Request::builder()
        .method("POST")
        .uri("/api/upload")
        .header("authorization", format!("Bearer {}", token))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(fake_body_vec))
        .unwrap();

    let fake_res = app.clone().oneshot(fake_req).await.unwrap();
    assert_eq!(fake_res.status(), StatusCode::BAD_REQUEST);

    // 3. Authenticated upload with valid PNG bytes -> Expect 201 CREATED
    let req = Request::builder()
        .method("POST")
        .uri("/api/upload")
        .header("authorization", format!("Bearer {}", token))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body_vec))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let file_url = json.get("url").and_then(|u| u.as_str()).unwrap();
    assert!(file_url.starts_with("/uploads/"));
    assert!(file_url.ends_with(".png"));

    // 4. Fetch the uploaded static file
    let static_req = Request::builder()
        .method("GET")
        .uri(file_url)
        .body(Body::empty())
        .unwrap();

    let static_res = app.oneshot(static_req).await.unwrap();
    assert_eq!(static_res.status(), StatusCode::OK);
}
