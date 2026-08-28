use serve::domain::storage::BlobStorage;
use serve::infrastructure::config::{
    AppConfig, AppEnvironment, DatabaseConfig, StorageConfig, StorageDriver, censor_database_url,
};
use serve::infrastructure::storage::StorageService;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.env, AppEnvironment::Development);
    assert!(config.server.enable_graphiql);
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.database.min_connections, 1);
    assert!(config.database.auto_migrate);
    assert_eq!(config.auth.jwt_expiration_days, 30);
    assert_eq!(config.storage.driver, StorageDriver::Local);
    assert_eq!(config.storage.local_path, "./uploads");
    assert!(config.cors.allows_any());
}

#[test]
fn test_socket_address_resolution() {
    let config = AppConfig::default();
    let addr = config.server.socket_addr();
    assert_eq!(
        addr,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    );
}

#[test]
fn test_database_url_censoring() {
    let url = "postgres://serve_user:secret_password_123@localhost:5432/serve_db";
    let censored = censor_database_url(url);
    assert!(!censored.contains("secret_password_123"));
    assert!(censored.contains("serve_user:****@localhost:5432/serve_db"));
}

#[test]
fn test_config_from_env_overrides() {
    unsafe {
        std::env::set_var("APP_ENV", "production");
        std::env::set_var("PORT", "9090");
        std::env::set_var("HOST", "0.0.0.0");
        std::env::set_var("DB_MAX_CONNECTIONS", "25");
        std::env::set_var("STORAGE_DRIVER", "s3");
        std::env::set_var("S3_BUCKET", "prod-ferro-media");
        std::env::set_var("S3_REGION", "ap-northeast-2");
        std::env::set_var("S3_PUBLIC_URL", "https://cdn.ferro.app");
        std::env::set_var(
            "CORS_ALLOWED_ORIGINS",
            "https://ferro.app,https://admin.ferro.app",
        );

        let config = AppConfig::from_env();
        assert_eq!(config.server.env, AppEnvironment::Production);
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.max_connections, 25);
        assert_eq!(config.storage.driver, StorageDriver::S3);
        assert_eq!(config.storage.s3_bucket, "prod-ferro-media");
        assert_eq!(config.storage.s3_region, "ap-northeast-2");
        assert_eq!(
            config.storage.s3_public_url.as_deref(),
            Some("https://cdn.ferro.app")
        );
        assert!(!config.cors.allows_any());
        assert_eq!(config.cors.allowed_origins.len(), 2);
        assert_eq!(config.cors.allowed_origins[0], "https://ferro.app");

        let storage_service = StorageService::from_config(&config.storage);
        assert_eq!(
            storage_service.get_url("avatar/user1.png"),
            "https://cdn.ferro.app/avatar/user1.png"
        );

        // Clean up
        std::env::remove_var("APP_ENV");
        std::env::remove_var("PORT");
        std::env::remove_var("HOST");
        std::env::remove_var("DB_MAX_CONNECTIONS");
        std::env::remove_var("STORAGE_DRIVER");
        std::env::remove_var("S3_BUCKET");
        std::env::remove_var("S3_REGION");
        std::env::remove_var("S3_PUBLIC_URL");
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}

#[test]
fn test_database_config_from_url() {
    let custom_url = "postgres://user:pass@custom-host:5433/custom_db";
    let db_config = DatabaseConfig::from_url(custom_url);
    assert_eq!(db_config.url, custom_url);
    assert_eq!(db_config.max_connections, 10);
    assert!(db_config.auto_migrate);
}

#[test]
fn test_storage_config_local_defaults() {
    let cfg = StorageConfig::default();
    let service = StorageService::from_config(&cfg);
    assert_eq!(service.get_url("img.jpg"), "/uploads/img.jpg");
}
