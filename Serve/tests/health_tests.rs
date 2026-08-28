use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serve::graphql::build_schema;
use serve::infrastructure::config::AppConfig;
use serve::infrastructure::db::postgres::Database;
use serve::routes::create_router;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_liveness_endpoints() {
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
    let schema = build_schema(
        db.clone(),
        config.auth.clone(),
        serve::infrastructure::pubsub::MessageBroker::default(),
    );
    let app = create_router(schema, db, config);

    // 1. GET /health
    let req = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Test /health/live
    let req_live = Request::builder()
        .uri("/health/live")
        .body(Body::empty())
        .unwrap();
    let res_live = app.clone().oneshot(req_live).await.unwrap();
    assert_eq!(res_live.status(), StatusCode::OK);

    // Test /health/ready (requires valid DB connection)
    let req_ready = Request::builder()
        .uri("/health/ready")
        .body(Body::empty())
        .unwrap();
    let res_ready = app.clone().oneshot(req_ready).await.unwrap();
    assert_eq!(res_ready.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res_ready.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ready");
    assert_eq!(json["database"], "connected");
    assert!(json["pool"]["size"].as_u64().is_some());

    // Test /metrics (Prometheus endpoint)
    let req_metrics = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let res_metrics = app.oneshot(req_metrics).await.unwrap();
    assert_eq!(res_metrics.status(), StatusCode::OK);
    let body_metrics = axum::body::to_bytes(res_metrics.into_body(), usize::MAX)
        .await
        .unwrap();
    let text_metrics = String::from_utf8(body_metrics.to_vec()).unwrap();
    assert!(text_metrics.contains("ferro_db_pool_connections_total"));
    assert!(text_metrics.contains("ferro_serve_info"));
}
