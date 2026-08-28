use crate::domain::errors::{DomainError, ErrorCode};
use crate::infrastructure::auth::hash_password;
use crate::infrastructure::config::{DatabaseConfig, DatabaseDriver};

use chrono::{Duration, Utc};
use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use std::str::FromStr;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
pub enum DatabaseBackend {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

#[derive(Clone)]
pub struct Database {
    backend: DatabaseBackend,
}

#[allow(dead_code)]
impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, DomainError> {
        match config.driver {
            DatabaseDriver::Sqlite => {
                info!(
                    target: "serve::db",
                    max_connections = config.max_connections,
                    min_connections = config.min_connections,
                    sqlite_path = %config.sqlite_path,
                    "Connecting to SQLite database pool..."
                );

                if config.sqlite_path != ":memory:"
                    && !config.sqlite_path.starts_with("sqlite::memory:")
                {
                    let path = std::path::Path::new(&config.sqlite_path);
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }

                let connect_url = if config.url.starts_with("sqlite:") {
                    config.url.clone()
                } else {
                    format!("sqlite://{}", config.sqlite_path)
                };

                let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&connect_url)
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Invalid SQLite connection URL");
                        DomainError::new(ErrorCode::ErrorInternal, "Invalid SQLite connection URL")
                    })?
                    .create_if_missing(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .foreign_keys(true);

                let pool = SqlitePoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
                    .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
                    .connect_with(opts)
                    .await
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Failed to connect to SQLite");
                        DomainError::new(
                            ErrorCode::ErrorInternal,
                            ErrorCode::ErrorInternal.as_str(),
                        )
                    })?;

                info!(target: "serve::db", "SQLite database connection pool established");
                let db = Self {
                    backend: DatabaseBackend::Sqlite(pool),
                };
                if config.auto_migrate {
                    db.init_tables().await?;
                }
                if config.seed_data {
                    db.seed_test_data().await?;
                }
                Ok(db)
            }
            DatabaseDriver::Postgres => {
                info!(
                    target: "serve::db",
                    max_connections = config.max_connections,
                    min_connections = config.min_connections,
                    "Connecting to PostgreSQL database pool..."
                );
                let pool = PgPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
                    .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
                    .connect(&config.url)
                    .await
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Failed to connect to PostgreSQL");
                        DomainError::new(
                            ErrorCode::ErrorInternal,
                            ErrorCode::ErrorInternal.as_str(),
                        )
                    })?;

                info!(target: "serve::db", "PostgreSQL database connection pool established");
                let db = Self {
                    backend: DatabaseBackend::Postgres(pool),
                };
                if config.auto_migrate {
                    db.init_tables().await?;
                }
                if config.seed_data {
                    db.seed_test_data().await?;
                }
                Ok(db)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn connect_url(database_url: &str) -> Result<Self, DomainError> {
        Self::connect(&DatabaseConfig::from_url(database_url)).await
    }

    #[allow(dead_code)]
    pub async fn connect_sqlite(file_path: &str) -> Result<Self, DomainError> {
        Self::connect(&DatabaseConfig::sqlite_mode(file_path)).await
    }

    pub fn backend(&self) -> &DatabaseBackend {
        &self.backend
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(&self.backend, DatabaseBackend::Sqlite(_))
    }

    pub fn is_postgres(&self) -> bool {
        matches!(&self.backend, DatabaseBackend::Postgres(_))
    }

    pub fn pg_pool(&self) -> &PgPool {
        match &self.backend {
            DatabaseBackend::Postgres(pool) => pool,
            DatabaseBackend::Sqlite(_) => {
                panic!("Attempted to access PgPool while running in SQLite mode")
            }
        }
    }

    pub fn sqlite_pool(&self) -> &SqlitePool {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => pool,
            DatabaseBackend::Postgres(_) => {
                panic!("Attempted to access SqlitePool while running in Postgres mode")
            }
        }
    }

    pub async fn init_tables(&self) -> Result<(), DomainError> {
        match &self.backend {
            DatabaseBackend::Postgres(pool) => {
                info!(target: "serve::db", "Running PostgreSQL database schema migrations...");
                let migrations: &[(&str, &str)] = &[
                    (
                        "0001_init",
                        include_str!("../../../migrations/0001_init.sql"),
                    ),
                    (
                        "0002_user_profile",
                        include_str!("../../../migrations/0002_user_profile.sql"),
                    ),
                    (
                        "0003_stories",
                        include_str!("../../../migrations/0003_stories.sql"),
                    ),
                    (
                        "0004_direct_messages",
                        include_str!("../../../migrations/0004_direct_messages.sql"),
                    ),
                    (
                        "0005_indexes",
                        include_str!("../../../migrations/0005_indexes.sql"),
                    ),
                    (
                        "0006_bookmarks",
                        include_str!("../../../migrations/0006_bookmarks.sql"),
                    ),
                    (
                        "0007_notifications",
                        include_str!("../../../migrations/0007_notifications.sql"),
                    ),
                    (
                        "0008_nested_comments",
                        include_str!("../../../migrations/0008_nested_comments.sql"),
                    ),
                    (
                        "0009_reposts_and_quotes",
                        include_str!("../../../migrations/0009_reposts_and_quotes.sql"),
                    ),
                    (
                        "0010_trust_and_safety",
                        include_str!("../../../migrations/0010_trust_and_safety.sql"),
                    ),
                    (
                        "0011_advanced_dm",
                        include_str!("../../../migrations/0011_advanced_dm.sql"),
                    ),
                    (
                        "0012_hashtags_mentions_fts",
                        include_str!("../../../migrations/0012_hashtags_mentions_fts.sql"),
                    ),
                    (
                        "0013_threaded_comments_and_interactions",
                        include_str!(
                            "../../../migrations/0013_threaded_comments_and_interactions.sql"
                        ),
                    ),
                    (
                        "0014_advanced_social_platform",
                        include_str!("../../../migrations/0014_advanced_social_platform.sql"),
                    ),
                    (
                        "0015_performance_indexes",
                        include_str!("../../../migrations/0015_performance_indexes.sql"),
                    ),
                ];

                for (name, sql) in migrations {
                    sqlx::raw_sql(*sql).execute(pool).await.map_err(|e| {
                        error!(target: "serve::db", error = %e, migration = %name, "Failed to execute migration");
                        DomainError::new(ErrorCode::ErrorInternal, ErrorCode::ErrorInternal.as_str())
                    })?;
                }

                info!(target: "serve::db", "PostgreSQL database schema migrations applied successfully");
            }
            DatabaseBackend::Sqlite(pool) => {
                info!(target: "serve::db", "Running SQLite schema initialization...");
                let schema_sql = include_str!("sqlite_schema.sql");
                sqlx::raw_sql(schema_sql).execute(pool).await.map_err(|e| {
                    error!(target: "serve::db", error = %e, "Failed to initialize SQLite schema");
                    DomainError::new(ErrorCode::ErrorInternal, ErrorCode::ErrorInternal.as_str())
                })?;
                info!(target: "serve::db", "SQLite schema initialized successfully");
            }
        }
        Ok(())
    }

    pub async fn seed_test_data(&self) -> Result<(), DomainError> {
        let user_count: i64 = match &self.backend {
            DatabaseBackend::Postgres(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(pool)
                .await
                .unwrap_or(0),
            DatabaseBackend::Sqlite(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(pool)
                .await
                .unwrap_or(0),
        };

        if user_count > 0 {
            info!(target: "serve::db", "Database already contains users ({user_count}). Skipping seed data insertion.");
            return Ok(());
        }

        info!(target: "serve::db", "Seeding initial test and dummy data...");
        let now = Utc::now();
        let expires_in_24h = now + Duration::hours(24);
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();
        let user3_id = Uuid::new_v4();
        let default_pw_hash = hash_password("password123").unwrap_or_default();

        match &self.backend {
            DatabaseBackend::Postgres(pool) => {
                let _ = sqlx::query(
                    "INSERT INTO users (id, username, email, password_hash, display_name, bio, avatar_url, header_image_url, location, website, created_at) VALUES
                    ($1, 'ferro_dev', 'ferro@example.com', $2, 'Ferro Architect', 'Building superfast Rust GraphQL SNS backends', 'https://api.dicebear.com/7.x/bottts/svg?seed=ferro', 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe', 'Seoul, Korea', 'https://ferro.dev', $3),
                    ($4, 'alex_coder', 'alex@example.com', $2, 'Alex Johnson', 'Full-stack engineer & Open Source enthusiast.', 'https://api.dicebear.com/7.x/bottts/svg?seed=alex', 'https://images.unsplash.com/photo-1550745165-9bc0b252726f', 'San Francisco, CA', 'https://alexcoder.io', $3),
                    ($5, 'design_guru', 'sophia@example.com', $2, 'Sophia Chen', 'UI/UX Designer turned Rust fanatic.', 'https://api.dicebear.com/7.x/bottts/svg?seed=sophia', 'https://images.unsplash.com/photo-1579546929518-9e396f3cc809', 'Tokyo, Japan', 'https://sophiadesign.com', $3)"
                )
                .bind(user1_id).bind(&default_pw_hash).bind(now)
                .bind(user2_id).bind(user3_id)
                .execute(pool).await;

                let post1_id = Uuid::new_v4();
                let post2_id = Uuid::new_v4();
                let post3_id = Uuid::new_v4();

                let _ = sqlx::query(
                    "INSERT INTO posts (id, author_id, content, created_at) VALUES
                    ($1, $2, 'Welcome to Ferro! Our Rust + Axum + async-graphql + PostgreSQL SNS is live!', $3),
                    ($4, $5, 'Async Rust with Axum & sqlx feels so blazingly fast. Loving PostgreSQL!', $3),
                    ($6, $7, 'Designing GraphQL schemas with Rust types and sqlx is a dream come true!', $3)"
                )
                .bind(post1_id).bind(user1_id).bind(now)
                .bind(post2_id).bind(user2_id)
                .bind(post3_id).bind(user3_id)
                .execute(pool).await;

                let _ = sqlx::query(
                    "INSERT INTO comments (id, post_id, author_id, content, created_at) VALUES
                    ($1, $2, $3, 'Congrats! The architecture looks amazingly clean!', $4),
                    ($5, $2, $6, 'So smooth! Looking forward to testing GraphQL queries.', $4)",
                )
                .bind(Uuid::new_v4())
                .bind(post1_id)
                .bind(user2_id)
                .bind(now)
                .bind(Uuid::new_v4())
                .bind(user3_id)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO follows (follower_id, followee_id, created_at) VALUES
                    ($1, $2, $3), ($4, $2, $3), ($2, $1, $3)",
                )
                .bind(user2_id)
                .bind(user1_id)
                .bind(now)
                .bind(user3_id)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO likes (user_id, post_id, created_at) VALUES
                    ($1, $2, $3), ($4, $2, $3), ($2, $5, $3)",
                )
                .bind(user2_id)
                .bind(post1_id)
                .bind(now)
                .bind(user3_id)
                .bind(post2_id)
                .execute(pool)
                .await;

                let story1_id = Uuid::new_v4();
                let story2_id = Uuid::new_v4();

                let _ = sqlx::query(
                    "INSERT INTO stories (id, author_id, media_url, caption, created_at, expires_at) VALUES
                    ($1, $2, 'https://images.unsplash.com/photo-1518770660439-4636190af475', 'Coding late night in Rust!', $3, $4),
                    ($5, $6, 'https://images.unsplash.com/photo-1534447677768-be436bb09401', 'Beautiful Tokyo sunset', $3, $4)"
                )
                .bind(story1_id).bind(user1_id).bind(now).bind(expires_in_24h)
                .bind(story2_id).bind(user3_id)
                .execute(pool).await;
            }
            DatabaseBackend::Sqlite(pool) => {
                let _ = sqlx::query(
                    "INSERT INTO users (id, username, email, password_hash, display_name, bio, avatar_url, header_image_url, location, website, created_at) VALUES
                    ($1, 'ferro_dev', 'ferro@example.com', $2, 'Ferro Architect', 'Building superfast Rust GraphQL SNS backends', 'https://api.dicebear.com/7.x/bottts/svg?seed=ferro', 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe', 'Seoul, Korea', 'https://ferro.dev', $3),
                    ($4, 'alex_coder', 'alex@example.com', $2, 'Alex Johnson', 'Full-stack engineer & Open Source enthusiast.', 'https://api.dicebear.com/7.x/bottts/svg?seed=alex', 'https://images.unsplash.com/photo-1550745165-9bc0b252726f', 'San Francisco, CA', 'https://alexcoder.io', $3),
                    ($5, 'design_guru', 'sophia@example.com', $2, 'Sophia Chen', 'UI/UX Designer turned Rust fanatic.', 'https://api.dicebear.com/7.x/bottts/svg?seed=sophia', 'https://images.unsplash.com/photo-1579546929518-9e396f3cc809', 'Tokyo, Japan', 'https://sophiadesign.com', $3)"
                )
                .bind(user1_id).bind(&default_pw_hash).bind(now)
                .bind(user2_id).bind(user3_id)
                .execute(pool).await;

                let post1_id = Uuid::new_v4();
                let post2_id = Uuid::new_v4();
                let post3_id = Uuid::new_v4();

                let _ = sqlx::query(
                    "INSERT INTO posts (id, author_id, content, created_at) VALUES
                    ($1, $2, 'Welcome to Ferro! Our Rust + Axum + async-graphql + SQLite SNS is live!', $3),
                    ($4, $5, 'Async Rust with Axum & sqlx feels so blazingly fast. Loving SQLite!', $3),
                    ($6, $7, 'Designing GraphQL schemas with Rust types and sqlx is a dream come true!', $3)"
                )
                .bind(post1_id).bind(user1_id).bind(now)
                .bind(post2_id).bind(user2_id)
                .bind(post3_id).bind(user3_id)
                .execute(pool).await;

                let _ = sqlx::query(
                    "INSERT INTO comments (id, post_id, author_id, content, created_at) VALUES
                    ($1, $2, $3, 'Congrats! The architecture looks amazingly clean!', $4),
                    ($5, $2, $6, 'So smooth! Looking forward to testing GraphQL queries.', $4)",
                )
                .bind(Uuid::new_v4())
                .bind(post1_id)
                .bind(user2_id)
                .bind(now)
                .bind(Uuid::new_v4())
                .bind(user3_id)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO follows (follower_id, followee_id, created_at) VALUES
                    ($1, $2, $3), ($4, $2, $3), ($2, $1, $3)",
                )
                .bind(user2_id)
                .bind(user1_id)
                .bind(now)
                .bind(user3_id)
                .execute(pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO likes (user_id, post_id, created_at) VALUES
                    ($1, $2, $3), ($4, $2, $3), ($2, $5, $3)",
                )
                .bind(user2_id)
                .bind(post1_id)
                .bind(now)
                .bind(user3_id)
                .bind(post2_id)
                .execute(pool)
                .await;

                let story1_id = Uuid::new_v4();
                let story2_id = Uuid::new_v4();

                let _ = sqlx::query(
                    "INSERT INTO stories (id, author_id, media_url, caption, created_at, expires_at) VALUES
                    ($1, $2, 'https://images.unsplash.com/photo-1518770660439-4636190af475', 'Coding late night in Rust!', $3, $4),
                    ($5, $6, 'https://images.unsplash.com/photo-1534447677768-be436bb09401', 'Beautiful Tokyo sunset', $3, $4)"
                )
                .bind(story1_id).bind(user1_id).bind(now).bind(expires_in_24h)
                .bind(story2_id).bind(user3_id)
                .execute(pool).await;
            }
        }

        info!(target: "serve::db", "Dummy test data seeded successfully!");
        Ok(())
    }
}
