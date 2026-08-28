pub mod graphql;
pub mod health;
pub mod upload;

use crate::graphql::SnsSchema;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::logging::http_trace::{
    create_http_trace_layer, create_propagate_request_id_layer, create_request_id_layer,
};
use crate::infrastructure::security::{RateLimiter, rate_limit_middleware};
use crate::infrastructure::storage::StorageService;

use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{get, post},
};
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

pub fn create_router(schema: SnsSchema, db: Database, config: AppConfig) -> Router {
    let ws_path = format!("{}/ws", config.server.graphql_path);
    let storage = StorageService::from_config(&config.storage);
    let rate_limiter = RateLimiter::new(
        config.rate_limit.max_requests,
        Duration::from_secs(config.rate_limit.window_secs),
    );

    let mut router = Router::new()
        // Health check & Metrics endpoints
        .route("/health", get(health::liveness_handler))
        .route("/health/live", get(health::liveness_handler))
        .route("/health/ready", get(health::readiness_handler))
        .route("/metrics", get(health::metrics_handler))
        // File Upload API endpoint
        .route("/api/upload", post(upload::upload_file_handler))
        // Static uploads directory serving
        .nest_service("/uploads", ServeDir::new(&config.storage.local_path))
        // GraphQL API HTTP & WebSocket endpoints
        .route(&config.server.graphql_path, post(graphql::graphql_handler))
        .route(&ws_path, get(graphql::graphql_ws_handler));

    // GraphiQL IDE endpoint (enabled in development or explicitly configured)
    if config.server.enable_graphiql {
        router = router
            .route("/", get(graphql::graphiql_handler))
            .route(&config.server.graphiql_path, get(graphql::graphiql_handler));
    }

    // Configure CORS layer
    let cors = if config.cors.allows_any() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = config
            .cors
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any)
    };

    router
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB request body limit
        .layer(Extension(schema))
        .layer(Extension(db))
        .layer(Extension(storage))
        .layer(Extension(rate_limiter))
        .layer(Extension(config))
        .layer(cors)
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("SAMEORIGIN"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(create_propagate_request_id_layer())
        .layer(create_http_trace_layer())
        .layer(create_request_id_layer())
}
