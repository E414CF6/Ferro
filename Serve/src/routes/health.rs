use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::infrastructure::db::postgres::{Database, DatabaseBackend};

/// Liveness probe: returns 200 OK if server process is running
pub async fn liveness_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "serve",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

/// Readiness probe: returns 200 OK with database connection pool diagnostics
pub async fn readiness_handler(Extension(db): Extension<Database>) -> impl IntoResponse {
    match db.backend() {
        DatabaseBackend::Postgres(pool) => {
            let ping_result = sqlx::query("SELECT 1").execute(pool).await;
            match ping_result {
                Ok(_) => {
                    let pool_size = pool.size();
                    let pool_idle = pool.num_idle();
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "ready",
                            "database": "connected",
                            "driver": "postgres",
                            "pool": {
                                "size": pool_size,
                                "idle": pool_idle,
                                "active": pool_size.saturating_sub(pool_idle as u32)
                            }
                        })),
                    )
                }
                Err(err) => {
                    tracing::error!(target: "serve::health", error = %err, "PostgreSQL readiness check failed");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "status": "unhealthy",
                            "database": "disconnected",
                            "error": err.to_string()
                        })),
                    )
                }
            }
        }
        DatabaseBackend::Sqlite(pool) => {
            let ping_result = sqlx::query("SELECT 1").execute(pool).await;
            match ping_result {
                Ok(_) => {
                    let pool_size = pool.size();
                    let pool_idle = pool.num_idle();
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "ready",
                            "database": "connected",
                            "driver": "sqlite",
                            "pool": {
                                "size": pool_size,
                                "idle": pool_idle,
                                "active": pool_size.saturating_sub(pool_idle as u32)
                            }
                        })),
                    )
                }
                Err(err) => {
                    tracing::error!(target: "serve::health", error = %err, "SQLite readiness check failed");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "status": "unhealthy",
                            "database": "disconnected",
                            "error": err.to_string()
                        })),
                    )
                }
            }
        }
    }
}

/// Prometheus metrics endpoint
pub async fn metrics_handler(Extension(db): Extension<Database>) -> impl IntoResponse {
    let (pool_size, pool_idle, pool_active) = match db.backend() {
        DatabaseBackend::Postgres(pool) => {
            let size = pool.size();
            let idle = pool.num_idle();
            (size, idle, size.saturating_sub(idle as u32))
        }
        DatabaseBackend::Sqlite(pool) => {
            let size = pool.size();
            let idle = pool.num_idle();
            (size, idle, size.saturating_sub(idle as u32))
        }
    };

    let metrics_text = format!(
        "# HELP ferro_serve_info Service build information\n\
         # TYPE ferro_serve_info gauge\n\
         ferro_serve_info{{version=\"{}\"}} 1\n\n\
         # HELP ferro_db_pool_connections_total Total database pool connections\n\
         # TYPE ferro_db_pool_connections_total gauge\n\
         ferro_db_pool_connections_total {}\n\n\
         # HELP ferro_db_pool_connections_idle Idle database connections\n\
         # TYPE ferro_db_pool_connections_idle gauge\n\
         ferro_db_pool_connections_idle {}\n\n\
         # HELP ferro_db_pool_connections_active Active database connections in use\n\
         # TYPE ferro_db_pool_connections_active gauge\n\
         ferro_db_pool_connections_active {}\n",
        env!("CARGO_PKG_VERSION"),
        pool_size,
        pool_idle,
        pool_active,
    );

    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics_text,
    )
}
