pub mod http_trace;

use std::env;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Supported logging output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Pretty colored logs for human-readable local development
    #[default]
    Pretty,
    /// Structured JSON logs for containerized / cloud environments (Docker, K8s, CloudWatch, Loki)
    Json,
    /// Compact single-line logs
    Compact,
}

impl LogFormat {
    pub fn from_env() -> Self {
        match env::var("LOG_FORMAT")
            .unwrap_or_else(|_| "pretty".to_string())
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            "compact" => LogFormat::Compact,
            _ => LogFormat::Pretty,
        }
    }
}

/// Initialize the global tracing subscriber with environment-based configuration.
pub fn init_logging() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
        "info,serve=debug,tower_http=info,sqlx=warn,async_graphql=info".to_string()
    });
    let format = LogFormat::from_env();
    init_logging_raw(&filter, format);
}

/// Initialize the global tracing subscriber with explicit filter and format
pub fn init_logging_raw(filter: &str, format: LogFormat) {
    let env_filter = EnvFilter::try_new(filter).unwrap_or_else(|_| {
        EnvFilter::new("info,serve=debug,tower_http=info,sqlx=warn,async_graphql=info")
    });

    let registry = tracing_subscriber::registry().with(env_filter);

    let init_result = match format {
        LogFormat::Json => {
            let json_layer = fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true)
                .with_file(true)
                .with_line_number(true)
                .with_target(true);
            registry.with(json_layer).try_init()
        }
        LogFormat::Compact => {
            let compact_layer = fmt::layer()
                .compact()
                .with_file(true)
                .with_line_number(true)
                .with_target(true);
            registry.with(compact_layer).try_init()
        }
        LogFormat::Pretty => {
            let pretty_layer = fmt::layer()
                .pretty()
                .with_file(false)
                .with_line_number(false)
                .with_target(true);
            registry.with(pretty_layer).try_init()
        }
    };

    if let Err(e) = init_result {
        // try_init returns Err if already initialized (e.g., in integration tests)
        eprintln!("Tracing subscriber initialization skipped or failed: {}", e);
    }
}
