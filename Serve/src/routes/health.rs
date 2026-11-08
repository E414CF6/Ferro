use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::infrastructure::config::AppConfig;
use crate::infrastructure::db::postgres::{Database, DatabaseBackend};
use crate::infrastructure::security::RateLimiter;

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

/// Readiness probe: returns 200 OK with database connection pool diagnostics and Redis connectivity
pub async fn readiness_handler(
    Extension(db): Extension<Database>,
    Extension(config): Extension<AppConfig>,
) -> impl IntoResponse {
    let (db_ok, pool_size, pool_idle, driver_name, db_err) = match db.backend() {
        DatabaseBackend::Postgres(pool) => {
            let ping_result = sqlx::query("SELECT 1").execute(pool).await;
            match ping_result {
                Ok(_) => (true, pool.size(), pool.num_idle(), "postgres", None),
                Err(err) => {
                    tracing::error!(target: "serve::health", error = %err, "PostgreSQL readiness check failed");
                    (
                        false,
                        pool.size(),
                        pool.num_idle(),
                        "postgres",
                        Some(err.to_string()),
                    )
                }
            }
        }
        DatabaseBackend::Sqlite(pool) => {
            let ping_result = sqlx::query("SELECT 1").execute(pool).await;
            match ping_result {
                Ok(_) => (true, pool.size(), pool.num_idle(), "sqlite", None),
                Err(err) => {
                    tracing::error!(target: "serve::health", error = %err, "SQLite readiness check failed");
                    (
                        false,
                        pool.size(),
                        pool.num_idle(),
                        "sqlite",
                        Some(err.to_string()),
                    )
                }
            }
        }
    };

    let redis_status = if config.redis.enabled {
        if let Ok(client) = redis::Client::open(config.redis.url.as_str()) {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let ping: Result<String, _> = redis::cmd("PING").query_async(&mut conn).await;
                if ping.is_ok() {
                    "connected"
                } else {
                    "disconnected"
                }
            } else {
                "disconnected"
            }
        } else {
            "invalid_url"
        }
    } else {
        "disabled"
    };

    if !db_ok {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "unhealthy",
                "database": "disconnected",
                "redis": redis_status,
                "error": db_err.unwrap_or_else(|| "Database ping failed".to_string())
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "ready",
            "database": "connected",
            "driver": driver_name,
            "redis": redis_status,
            "pool": {
                "size": pool_size,
                "idle": pool_idle,
                "active": pool_size.saturating_sub(pool_idle as u32)
            }
        })),
    )
}

/// Prometheus metrics endpoint (optionally protected by METRICS_AUTH_TOKEN)
pub async fn metrics_handler(
    Extension(db): Extension<Database>,
    Extension(rate_limiter): Extension<RateLimiter>,
    Extension(config): Extension<AppConfig>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(ref expected_token) = config.server.metrics_token {
        let auth_valid = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| {
                let token = h
                    .strip_prefix("Bearer ")
                    .or_else(|| h.strip_prefix("bearer "))
                    .unwrap_or(h);
                token.trim() == expected_token
            })
            .unwrap_or(false);

        if !auth_valid {
            return (
                StatusCode::UNAUTHORIZED,
                [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
                "# Error: Unauthorized access to /metrics (invalid or missing bearer token)\n"
                    .to_string(),
            );
        }
    }

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

    let redis_connected = if rate_limiter.is_redis_connected() {
        1
    } else {
        0
    };

    let reqs_2xx = crate::infrastructure::metrics::HTTP_METRICS
        .requests_2xx
        .load(std::sync::atomic::Ordering::Relaxed);
    let reqs_3xx = crate::infrastructure::metrics::HTTP_METRICS
        .requests_3xx
        .load(std::sync::atomic::Ordering::Relaxed);
    let reqs_4xx = crate::infrastructure::metrics::HTTP_METRICS
        .requests_4xx
        .load(std::sync::atomic::Ordering::Relaxed);
    let reqs_5xx = crate::infrastructure::metrics::HTTP_METRICS
        .requests_5xx
        .load(std::sync::atomic::Ordering::Relaxed);
    let duration_sec = crate::infrastructure::metrics::HTTP_METRICS
        .total_duration_micros
        .load(std::sync::atomic::Ordering::Relaxed) as f64
        / 1_000_000.0;

    let metrics_text = format!(
        "# HELP ferro_serve_info Service build information\n\
         # TYPE ferro_serve_info gauge\n\
         ferro_serve_info{{version=\"{}\"}} 1\n\n\
         # HELP ferro_http_requests_total Total HTTP requests processed\n\
         # TYPE ferro_http_requests_total counter\n\
         ferro_http_requests_total{{status=\"2xx\"}} {}\n\
         ferro_http_requests_total{{status=\"3xx\"}} {}\n\
         ferro_http_requests_total{{status=\"4xx\"}} {}\n\
         ferro_http_requests_total{{status=\"5xx\"}} {}\n\n\
         # HELP ferro_http_request_duration_seconds_total Total duration of processed HTTP requests in seconds\n\
         # TYPE ferro_http_request_duration_seconds_total counter\n\
         ferro_http_request_duration_seconds_total {:.6}\n\n\
         # HELP ferro_db_pool_connections_total Total database pool connections\n\
         # TYPE ferro_db_pool_connections_total gauge\n\
         ferro_db_pool_connections_total {}\n\n\
         # HELP ferro_db_pool_connections_idle Idle database connections\n\
         # TYPE ferro_db_pool_connections_idle gauge\n\
         ferro_db_pool_connections_idle {}\n\n\
         # HELP ferro_db_pool_connections_active Active database connections in use\n\
         # TYPE ferro_db_pool_connections_active gauge\n\
         ferro_db_pool_connections_active {}\n\n\
         # HELP ferro_redis_connected Redis cluster connectivity state (1 = connected, 0 = standalone/disconnected)\n\
         # TYPE ferro_redis_connected gauge\n\
         ferro_redis_connected {}\n\n\
         # HELP ferro_rate_limit_max_requests Maximum configured requests per window\n\
         # TYPE ferro_rate_limit_max_requests gauge\n\
         ferro_rate_limit_max_requests {}\n",
        env!("CARGO_PKG_VERSION"),
        reqs_2xx,
        reqs_3xx,
        reqs_4xx,
        reqs_5xx,
        duration_sec,
        pool_size,
        pool_idle,
        pool_active,
        redis_connected,
        rate_limiter.max_requests(),
    );

    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics_text,
    )
}
