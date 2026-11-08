use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Atomic HTTP metrics collector for Prometheus scraping
#[derive(Default)]
pub struct HttpMetrics {
    pub requests_2xx: AtomicU64,
    pub requests_3xx: AtomicU64,
    pub requests_4xx: AtomicU64,
    pub requests_5xx: AtomicU64,
    pub total_duration_micros: AtomicU64,
}

impl HttpMetrics {
    pub const fn new() -> Self {
        Self {
            requests_2xx: AtomicU64::new(0),
            requests_3xx: AtomicU64::new(0),
            requests_4xx: AtomicU64::new(0),
            requests_5xx: AtomicU64::new(0),
            total_duration_micros: AtomicU64::new(0),
        }
    }

    pub fn record(&self, status_code: u16, latency: Duration) {
        match status_code {
            200..=299 => {
                self.requests_2xx.fetch_add(1, Ordering::Relaxed);
            }
            300..=399 => {
                self.requests_3xx.fetch_add(1, Ordering::Relaxed);
            }
            400..=499 => {
                self.requests_4xx.fetch_add(1, Ordering::Relaxed);
            }
            500..=599 => {
                self.requests_5xx.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
        self.total_duration_micros
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }
}

pub static HTTP_METRICS: HttpMetrics = HttpMetrics::new();
