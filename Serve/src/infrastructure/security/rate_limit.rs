use axum::{
    extract::{Extension, Request},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
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

/// Thread-safe hybrid (Redis distributed + in-memory fallback) sliding window rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window_duration: Duration,
    clients: Arc<RwLock<HashMap<String, RateBucket>>>,
    redis_client: Option<redis::Client>,
}

impl RateLimiter {
    /// Create new in-memory rate limiter: `max_requests` per `window_duration`
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self::with_redis_opt(max_requests, window_duration, None)
    }

    /// Create distributed Redis-backed rate limiter with in-memory fallback
    pub fn with_redis(max_requests: usize, window_duration: Duration, redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).ok();
        Self::with_redis_opt(max_requests, window_duration, client)
    }

    pub fn with_redis_opt(
        max_requests: usize,
        window_duration: Duration,
        redis_client: Option<redis::Client>,
    ) -> Self {
        let limiter = Self {
            max_requests,
            window_duration,
            clients: Arc::new(RwLock::new(HashMap::new())),
            redis_client,
        };

        // Periodic background cleanup of stale entries in local in-memory fallback map
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
        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let redis_key = format!("ferro:ratelimit:{}", key);
                let now_ms = Utc::now().timestamp_millis();
                let window_ms = self.window_duration.as_millis() as i64;
                let clear_before = now_ms - window_ms;
                let max_req = self.max_requests as i64;

                let script = redis::Script::new(
                    r#"
                    local key = KEYS[1]
                    local now = tonumber(ARGV[1])
                    local clear_before = tonumber(ARGV[2])
                    local window_secs = tonumber(ARGV[3])
                    local max_req = tonumber(ARGV[4])

                    redis.call('ZREMRANGEBYSCORE', key, '-inf', clear_before)
                    local current = redis.call('ZCARD', key)
                    if current < max_req then
                        redis.call('ZADD', key, now, now .. ':' .. redis.call('INCR', key .. ':seq'))
                        redis.call('EXPIRE', key, window_secs * 2)
                        return {1, max_req - current - 1, window_secs}
                    else
                        return {0, 0, window_secs}
                    end
                "#,
                );

                let res: Result<(i64, i64, i64), _> = script
                    .key(&redis_key)
                    .arg(now_ms)
                    .arg(clear_before)
                    .arg(self.window_duration.as_secs() as i64)
                    .arg(max_req)
                    .invoke_async(&mut conn)
                    .await;

                if let Ok((allowed, remaining, reset_after)) = res {
                    return (
                        allowed == 1,
                        remaining.max(0) as usize,
                        reset_after.max(1) as u64,
                    );
                }
            }
        }

        self.check_in_memory(key).await
    }

    /// In-memory sliding window check
    pub async fn check_in_memory(&self, key: &str) -> (bool, usize, u64) {
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

    pub fn is_redis_connected(&self) -> bool {
        self.redis_client.is_some()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Default: 120 requests per minute per IP
        Self::new(120, Duration::from_secs(60))
    }
}

use axum::extract::ConnectInfo;

/// Helper to extract client IP from HTTP headers or ConnectInfo, respecting trust_proxy configuration
pub fn extract_client_ip(
    headers: &HeaderMap,
    connect_info: Option<&SocketAddr>,
    trust_proxy: bool,
) -> String {
    if trust_proxy {
        if let Some(cf_ip) = headers
            .get("cf-connecting-ip")
            .and_then(|v| v.to_str().ok())
        {
            let trimmed = cf_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        if let Some(xfwd) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first_ip) = xfwd.split(',').next() {
                let trimmed = first_ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }

        if let Some(real_ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
            let trimmed = real_ip.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
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
    if path == "/health" || path == "/health/live" || path == "/health/ready" || path == "/metrics"
    {
        return next.run(req).await;
    }

    if !config.rate_limit.enabled {
        return next.run(req).await;
    }

    let headers = req.headers();
    let connect_info = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0);
    let client_ip = extract_client_ip(
        headers,
        connect_info.as_ref(),
        config.rate_limit.trust_proxy,
    );

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
