use axum::{
    extract::{Extension, Request},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::warn;

use crate::infrastructure::config::AppConfig;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct RateBucket {
    count: usize,
    window_start: Instant,
}

/// Thread-safe in-memory sliding window rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window_duration: Duration,
    clients: Arc<RwLock<HashMap<String, RateBucket>>>,
}

impl RateLimiter {
    /// Create new rate limiter: `max_requests` per `window_duration`
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        let limiter = Self {
            max_requests,
            window_duration,
            clients: Arc::new(RwLock::new(HashMap::new())),
        };

        // Periodic background cleanup of stale entries every 60 seconds
        let clients_clone = limiter.clients.clone();
        let window = window_duration;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut map = clients_clone.write().await;
                let now = Instant::now();
                map.retain(|_, bucket| now.duration_since(bucket.window_start) < window * 2);
            }
        });

        limiter
    }

    /// Check if a request from the given key (e.g. IP address or client ID) is allowed.
    /// Returns `(is_allowed, remaining_requests, reset_after_seconds)`
    pub async fn check(&self, key: &str) -> (bool, usize, u64) {
        let now = Instant::now();
        let mut map = self.clients.write().await;

        let bucket = map.entry(key.to_string()).or_insert_with(|| RateBucket {
            count: 0,
            window_start: now,
        });

        let elapsed = now.duration_since(bucket.window_start);
        if elapsed >= self.window_duration {
            bucket.count = 1;
            bucket.window_start = now;
            (
                true,
                self.max_requests.saturating_sub(1),
                self.window_duration.as_secs(),
            )
        } else if bucket.count < self.max_requests {
            bucket.count += 1;
            let remaining = self.max_requests.saturating_sub(bucket.count);
            let reset_after = self.window_duration.saturating_sub(elapsed).as_secs();
            (true, remaining, reset_after)
        } else {
            let reset_after = self.window_duration.saturating_sub(elapsed).as_secs();
            warn!(
                target: "serve::security::ratelimit",
                key = %key,
                limit = self.max_requests,
                "Rate limit threshold exceeded"
            );
            (false, 0, reset_after.max(1))
        }
    }

    pub fn max_requests(&self) -> usize {
        self.max_requests
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Default: 120 requests per minute per IP
        Self::new(120, Duration::from_secs(60))
    }
}

/// Helper to extract client IP from HTTP headers or ConnectInfo
pub fn extract_client_ip(headers: &HeaderMap, connect_info: Option<&SocketAddr>) -> String {
    if let Some(cf_ip) = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
    {
        return cf_ip.trim().to_string();
    }

    if let Some(xfwd) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_ip) = xfwd.split(',').next() {
            let trimmed = first_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return real_ip.trim().to_string();
    }

    if let Some(addr) = connect_info {
        return addr.ip().to_string();
    }

    "127.0.0.1".to_string()
}

static HEADER_RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
static HEADER_RATELIMIT_REMAINING: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
static HEADER_RATELIMIT_RESET: HeaderName = HeaderName::from_static("x-ratelimit-reset");

/// Axum middleware for IP-based sliding window rate limiting
pub async fn rate_limit_middleware(
    Extension(rate_limiter): Extension<RateLimiter>,
    Extension(config): Extension<AppConfig>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Bypass rate limiting for health check, readiness and metrics endpoints
    if path == "/health"
        || path == "/health/live"
        || path == "/health/ready"
        || path == "/metrics"
    {
        return next.run(req).await;
    }

    if !config.rate_limit.enabled {
        return next.run(req).await;
    }

    let headers = req.headers();
    let client_ip = extract_client_ip(headers, None);

    let (allowed, remaining, reset_after) = rate_limiter.check(&client_ip).await;

    if !allowed {
        let body = Json(json!({
            "error": "RATE_LIMIT_EXCEEDED",
            "message": "Too many requests. Please try again later.",
            "retry_after_seconds": reset_after
        }));

        let mut res = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
        let headers_mut = res.headers_mut();

        if let Ok(val) = HeaderValue::from_str(&reset_after.to_string()) {
            headers_mut.insert(header::RETRY_AFTER, val);
        }
        if let Ok(val) = HeaderValue::from_str(&rate_limiter.max_requests().to_string()) {
            headers_mut.insert(HEADER_RATELIMIT_LIMIT.clone(), val);
        }
        if let Ok(val) = HeaderValue::from_str(&remaining.to_string()) {
            headers_mut.insert(HEADER_RATELIMIT_REMAINING.clone(), val);
        }
        if let Ok(val) = HeaderValue::from_str(&reset_after.to_string()) {
            headers_mut.insert(HEADER_RATELIMIT_RESET.clone(), val);
        }

        return res;
    }

    let mut response = next.run(req).await;
    let headers_mut = response.headers_mut();

    if let Ok(val) = HeaderValue::from_str(&rate_limiter.max_requests().to_string()) {
        headers_mut.insert(HEADER_RATELIMIT_LIMIT.clone(), val);
    }
    if let Ok(val) = HeaderValue::from_str(&remaining.to_string()) {
        headers_mut.insert(HEADER_RATELIMIT_REMAINING.clone(), val);
    }
    if let Ok(val) = HeaderValue::from_str(&reset_after.to_string()) {
        headers_mut.insert(HEADER_RATELIMIT_RESET.clone(), val);
    }

    response
}
