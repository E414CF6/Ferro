use crate::infrastructure::logging::LogFormat;

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tracing::{info, warn};

/// Application deployment environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppEnvironment {
    #[default]
    Development,
    Production,
    Staging,
    Test,
}

impl AppEnvironment {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEnvironment::Development => "development",
            AppEnvironment::Production => "production",
            AppEnvironment::Staging => "staging",
            AppEnvironment::Test => "test",
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, AppEnvironment::Production)
    }
}

impl FromStr for AppEnvironment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "prod" | "production" => Ok(AppEnvironment::Production),
            "stage" | "staging" => Ok(AppEnvironment::Staging),
            "test" | "testing" => Ok(AppEnvironment::Test),
            _ => Ok(AppEnvironment::Development),
        }
    }
}

/// Server network and endpoint configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub env: AppEnvironment,
    pub enable_graphiql: bool,
    pub enable_introspection: bool,
    pub graphiql_path: String,
    pub graphql_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            env: AppEnvironment::Development,
            enable_graphiql: true,
            enable_introspection: true,
            graphiql_path: "/graphiql".to_string(),
            graphql_path: "/graphql".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn socket_addr(&self) -> SocketAddr {
        let ip: IpAddr = self
            .host
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        SocketAddr::new(ip, self.port)
    }
}

/// Supported database drivers / storage modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatabaseDriver {
    #[default]
    Postgres,
    Sqlite,
}

#[allow(dead_code)]
impl DatabaseDriver {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseDriver::Postgres => "postgres",
            DatabaseDriver::Sqlite => "sqlite",
        }
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(self, DatabaseDriver::Sqlite)
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DatabaseDriver::Postgres)
    }
}

impl FromStr for DatabaseDriver {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "sqlite" | "sqlite3" | "local" | "file" => Ok(DatabaseDriver::Sqlite),
            _ => Ok(DatabaseDriver::Postgres),
        }
    }
}

/// PostgreSQL / SQLite database and connection pool configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseConfig {
    pub driver: DatabaseDriver,
    pub url: String,
    pub sqlite_path: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub auto_migrate: bool,
    pub seed_data: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            driver: DatabaseDriver::Postgres,
            url: "postgres://serve:password@localhost:5432/serve".to_string(),
            sqlite_path: "./serve.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 10,
            acquire_timeout_secs: 15,
            idle_timeout_secs: 600,
            auto_migrate: true,
            seed_data: false,
        }
    }
}

impl DatabaseConfig {
    pub fn from_url(url: &str) -> Self {
        let is_sqlite = url.starts_with("sqlite://")
            || url.starts_with("sqlite:")
            || url.ends_with(".db")
            || url.ends_with(".sqlite");

        let driver = if is_sqlite {
            DatabaseDriver::Sqlite
        } else {
            DatabaseDriver::Postgres
        };

        let sqlite_path = if let Some(stripped) = url.strip_prefix("sqlite://") {
            stripped.to_string()
        } else if let Some(stripped) = url.strip_prefix("sqlite:") {
            stripped.to_string()
        } else if url.ends_with(".db") || url.ends_with(".sqlite") {
            url.to_string()
        } else {
            "./serve.db".to_string()
        };

        Self {
            driver,
            url: url.to_string(),
            sqlite_path,
            ..Default::default()
        }
    }

    pub fn sqlite_mode(path: &str) -> Self {
        let url = if path.starts_with("sqlite:") {
            path.to_string()
        } else {
            format!("sqlite://{}", path)
        };
        Self {
            driver: DatabaseDriver::Sqlite,
            url,
            sqlite_path: path.to_string(),
            ..Default::default()
        }
    }
}

/// Supported storage drivers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageDriver {
    #[default]
    Local,
    S3,
}

impl StorageDriver {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageDriver::Local => "local",
            StorageDriver::S3 => "s3",
        }
    }
}

impl FromStr for StorageDriver {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "s3" | "r2" | "minio" | "gcs" => Ok(StorageDriver::S3),
            _ => Ok(StorageDriver::Local),
        }
    }
}

/// Blob / Object storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub driver: StorageDriver,
    pub local_path: String,
    pub local_base_url: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_endpoint: Option<String>,
    pub s3_public_url: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            driver: StorageDriver::Local,
            local_path: "./uploads".to_string(),
            local_base_url: "/uploads".to_string(),
            s3_bucket: "serve-media".to_string(),
            s3_region: "auto".to_string(),
            s3_endpoint: None,
            s3_public_url: None,
            s3_access_key: None,
            s3_secret_key: None,
        }
    }
}

/// Authentication and JWT security configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_days: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "ferro-development-secret-key-do-not-use-in-production-32bytes!"
                .to_string(),
            jwt_expiration_days: 30,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: usize,
    pub window_secs: u64,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 120,
            window_secs: 60,
            enabled: true,
        }
    }
}

/// Cross-Origin Resource Sharing (CORS) configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
        }
    }
}

impl CorsConfig {
    pub fn allows_any(&self) -> bool {
        self.allowed_origins.iter().any(|o| o == "*")
    }
}

/// Tracing and structured logging configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoggingConfig {
    pub format: LogFormat,
    pub filter: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Pretty,
            filter: "info,serve=debug,tower_http=info,sqlx=warn,async_graphql=info".to_string(),
        }
    }
}

/// Unified, production-grade application configuration
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    /// Load configuration from environment variables with graceful defaults and .env file support
    pub fn from_env() -> Self {
        // Load .env if present
        if let Err(e) = dotenvy::dotenv() {
            tracing::trace!("No .env file loaded: {}", e);
        }

        let env = env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .parse::<AppEnvironment>()
            .unwrap_or_default();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let enable_graphiql = env::var("ENABLE_GRAPHIQL")
            .map(|v| v == "true" || v == "1")
            .unwrap_or_else(|_| !env.is_production());

        let enable_introspection = env::var("ENABLE_INTROSPECTION")
            .map(|v| v == "true" || v == "1")
            .unwrap_or_else(|_| !env.is_production());

        let graphiql_path = env::var("GRAPHIQL_PATH").unwrap_or_else(|_| "/graphiql".to_string());
        let graphql_path = env::var("GRAPHQL_PATH").unwrap_or_else(|_| "/graphql".to_string());

        let server = ServerConfig {
            host,
            port,
            env,
            enable_graphiql,
            enable_introspection,
            graphiql_path,
            graphql_path,
        };

        let driver_env = env::var("DATABASE_DRIVER")
            .or_else(|_| env::var("DATABASE_MODE"))
            .or_else(|_| env::var("DATABASE_TYPE"))
            .unwrap_or_default();

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://serve:password@localhost:5432/serve".to_string());

        let is_sqlite = driver_env.eq_ignore_ascii_case("sqlite")
            || driver_env.eq_ignore_ascii_case("sqlite3")
            || database_url.starts_with("sqlite://")
            || database_url.starts_with("sqlite:")
            || database_url.ends_with(".db")
            || database_url.ends_with(".sqlite");

        let driver = if is_sqlite {
            DatabaseDriver::Sqlite
        } else {
            DatabaseDriver::Postgres
        };

        let sqlite_path = env::var("SQLITE_PATH").unwrap_or_else(|_| {
            if let Some(stripped) = database_url.strip_prefix("sqlite://") {
                stripped.to_string()
            } else if let Some(stripped) = database_url.strip_prefix("sqlite:") {
                stripped.to_string()
            } else if database_url.ends_with(".db") || database_url.ends_with(".sqlite") {
                database_url.clone()
            } else {
                "./serve.db".to_string()
            }
        });

        let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let min_connections: u32 = env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let connect_timeout_secs: u64 = env::var("DB_CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let acquire_timeout_secs: u64 = env::var("DB_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);
        let idle_timeout_secs: u64 = env::var("DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);
        let auto_migrate = env::var("DB_AUTO_MIGRATE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        let seed_data = env::var("DB_SEED_DATA")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let database = DatabaseConfig {
            driver,
            url: database_url,
            sqlite_path,
            max_connections,
            min_connections,
            connect_timeout_secs,
            acquire_timeout_secs,
            idle_timeout_secs,
            auto_migrate,
            seed_data,
        };

        let storage_driver = env::var("STORAGE_DRIVER")
            .unwrap_or_default()
            .parse::<StorageDriver>()
            .unwrap_or_default();
        let local_path = env::var("STORAGE_LOCAL_PATH").unwrap_or_else(|_| "./uploads".to_string());
        let local_base_url =
            env::var("STORAGE_LOCAL_BASE_URL").unwrap_or_else(|_| "/uploads".to_string());
        let s3_bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "serve-media".to_string());
        let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string());
        let s3_endpoint = env::var("S3_ENDPOINT").ok();
        let s3_public_url = env::var("S3_PUBLIC_URL").ok();
        let s3_access_key = env::var("S3_ACCESS_KEY_ID")
            .or_else(|_| env::var("AWS_ACCESS_KEY_ID"))
            .ok();
        let s3_secret_key = env::var("S3_SECRET_ACCESS_KEY")
            .or_else(|_| env::var("AWS_SECRET_ACCESS_KEY"))
            .ok();

        let storage = StorageConfig {
            driver: storage_driver,
            local_path,
            local_base_url,
            s3_bucket,
            s3_region,
            s3_endpoint,
            s3_public_url,
            s3_access_key,
            s3_secret_key,
        };

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            "ferro-development-secret-key-do-not-use-in-production-32bytes!".to_string()
        });

        if env.is_production() && (jwt_secret.contains("development") || jwt_secret.len() < 32) {
            warn!(
                target: "serve::config",
                "SECURITY WARNING: Default or weak JWT_SECRET is being used in PRODUCTION! Please set a strong JWT_SECRET environment variable."
            );
        }

        let jwt_expiration_days: i64 = env::var("JWT_EXPIRATION_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let auth = AuthConfig {
            jwt_secret,
            jwt_expiration_days,
        };

        let rate_limit_max: usize = env::var("RATE_LIMIT_MAX_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);
        let rate_limit_window: u64 = env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let rate_limit_enabled = env::var("RATE_LIMIT_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let rate_limit = RateLimitConfig {
            max_requests: rate_limit_max,
            window_secs: rate_limit_window,
            enabled: rate_limit_enabled,
        };

        let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cors = CorsConfig { allowed_origins };

        let logging_format = LogFormat::from_env();
        let logging_filter = env::var("RUST_LOG").unwrap_or_else(|_| {
            "info,serve=debug,tower_http=info,sqlx=warn,async_graphql=info".to_string()
        });

        let logging = LoggingConfig {
            format: logging_format,
            filter: logging_filter,
        };

        Self {
            server,
            database,
            storage,
            auth,
            rate_limit,
            cors,
            logging,
        }
    }

    pub fn print_summary(&self) {
        info!(
            target: "serve::init",
            environment = %self.server.env.as_str(),
            host = %self.server.host,
            port = %self.server.port,
            database_driver = %self.database.driver.as_str(),
            database_url = %censor_database_url(&self.database.url),
            sqlite_path = %self.database.sqlite_path,
            storage_driver = %self.storage.driver.as_str(),
            max_connections = %self.database.max_connections,
            graphiql_enabled = %self.server.enable_graphiql,
            introspection_enabled = %self.server.enable_introspection,
            rate_limit_enabled = %self.rate_limit.enabled,
            rate_limit = %format!("{}/{}s", self.rate_limit.max_requests, self.rate_limit.window_secs),
            "Application configuration initialized"
        );
    }
}

/// Mask password in database URL for safe logging
pub fn censor_database_url(url: &str) -> String {
    let Some(at_idx) = url.find('@') else {
        return url.to_string();
    };
    let Some(colon_idx) = url[..at_idx].rfind(':') else {
        return url.to_string();
    };
    if colon_idx > 0 && &url[colon_idx - 1..colon_idx] != "/" {
        format!("{}:****{}", &url[..colon_idx], &url[at_idx..])
    } else {
        url.to_string()
    }
}
