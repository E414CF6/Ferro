use crate::domain::errors::{DomainError, ErrorCode};
use crate::infrastructure::auth::hash_password;
use crate::infrastructure::config::{DatabaseConfig, DatabaseDriver};

use chrono::{DateTime, Duration, Utc};
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
                    .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                    .busy_timeout(std::time::Duration::from_secs(5))
                    .foreign_keys(true);

                let pool = SqlitePoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
                    .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
                    .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_secs))
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
                    max_lifetime = config.max_lifetime_secs,
                    "Connecting to PostgreSQL database pool..."
                );
                let pool = PgPoolOptions::new()
                    .max_connections(config.max_connections)
                    .min_connections(config.min_connections)
                    .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
                    .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
                    .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_secs))
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
                    (
                        "0016_wiki_map",
                        include_str!("../../../migrations/0016_wiki_map.sql"),
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

        self.seed_wiki_data(now).await?;

        info!(target: "serve::db", "Dummy test data seeded successfully!");
        Ok(())
    }

    pub async fn seed_wiki_data(&self, now: DateTime<Utc>) -> Result<(), DomainError> {
        let wiki_count: i64 = match &self.backend {
            DatabaseBackend::Postgres(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM wiki_articles")
                .fetch_one(pool)
                .await
                .unwrap_or(0),
            DatabaseBackend::Sqlite(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM wiki_articles")
                .fetch_one(pool)
                .await
                .unwrap_or(0),
        };

        if wiki_count > 0 {
            info!(target: "serve::db", "Database already contains wiki articles ({wiki_count}). Skipping wiki seed.");
            return Ok(());
        }

        info!(target: "serve::db", "Seeding initial Wiki articles and view logs...");

        let articles = [
            (
                "경복궁 (Gyeongbokgung)",
                "gyeongbokgung",
                "조선 왕조 제일의 법궁으로, 1395년 태조 이성계에 의해 창건되었습니다.",
                "# 경복궁 (景福宮)\n\n**경복궁**은 대한민국 서울특별시 종로구 사직로에 있는 조선 왕조의 법궁(法宮)입니다.\n\n## 역사\n1395년(태조 4년)에 창건되었으며, '큰 복을 누리리라'는 뜻의 시경 구절에서 이름을 따왔습니다.\n\n## 주요 전각\n- **근정전(勤政殿)**: 국가의 중대한 의식을 거행하던 정전\n- **경회루(慶會樓)**: 외국 사신을 접대하거나 연회를 베풀던 누각\n- **향원정(香遠亭)**: 후원의 연못 가운데 세워진 아름다운 육각형 정자",
                37.579617,
                126.977041,
                16.0,
                "궁궐",
                r#"["궁궐","조선","문화유산","한복체험","산책"]"#,
                "위키여행가",
                60i64,
            ),
            (
                "N서울타워 (N Seoul Tower)",
                "n-seoul-tower",
                "남산 정상에 위치한 서울의 랜드마크이자 전파탑 및 복합문화공간.",
                "# N서울타워 (남산타워)\n\n**N서울타워**는 서울 남산 정상에 위치한 종합 전파탑이자 전망대입니다. 높이는 해발 479.7m에 달합니다.\n\n## 특징\n- **디지털 전망대**: 서울 360도 전경을 파노라마로 감상할 수 있습니다.\n- **사랑의 자물쇠**: 남산 타워의 명물입니다.",
                37.551169,
                126.988227,
                15.0,
                "야경",
                r#"["야경","남산","전망대","데이트코스","산책"]"#,
                "서울산책자",
                51i64,
            ),
            (
                "여의도 한강공원 (Yeouido Hangang Park)",
                "yeouido-hangang-park",
                "정치, 금융의 중심지 여의도에 맞닿아 있는 대표적인 수변 도심 공원.",
                "# 여의도 한강공원\n\n**여의도 한강공원**은 여의도동 한강변에 위치한 공원으로, 서울 시민들이 가장 많이 찾는 휴식처 중 하나입니다.\n\n## 주요 행사 & 시설\n- **봄 벚꽃축제**: 윤중로를 따라 펼쳐지는 벚꽃 명소\n- **서울세계불꽃축제**: 매년 가을 전 세계 팀이 참가하는 화려한 불꽃쇼\n- **물빛광장**: 여름철 물놀이와 산책 공간",
                37.52843,
                126.93285,
                15.0,
                "피크닉",
                r#"["피크닉","한강","자전거","벚꽃축제","산책"]"#,
                "도시탐험가",
                42i64,
            ),
            (
                "DDP (동대문디자인플라자)",
                "ddp",
                "자하 하디드가 설계한 세계 최대 규모의 3차원 비정형 건축물이자 복합 문화공간.",
                "# DDP (동대문디자인플라자)\n\n**동대문디자인플라자(DDP)**는 세계적인 건축가 **자하 하디드(Zaha Hadid)**가 설계한 미래지향적 곡선 건축물입니다.\n\n## 공간 구성\n- **알림터**: 패션쇼, 신제품 런칭 등 컨벤션 행사\n- **배움터**: 디자인 박물관 및 둘레길 전시 공간",
                37.56658,
                127.00918,
                16.0,
                "디자인",
                r#"["디자인","건축","전시회","야경","패션"]"#,
                "아키텍트",
                33i64,
            ),
            (
                "덕수궁 돌담길 (Deoksugung Stonewall)",
                "deoksugung-stonewall",
                "고풍스러운 돌담과 가로수가 어우러진 서울의 대표적인 낭만 산책로.",
                "# 덕수궁 돌담길\n\n서울 시청 맞은편 덕수궁 대한문 옆에서 시작하여 정동교회와 서울시립미술관으로 이어지는 유서 깊은 산책길입니다.\n\n## 매력 포인트\n- **사계절 풍경**: 가을 단풍 터널과 겨울 설경이 특히 아름답습니다.\n- **정동 역사 산책**: 근대 문화유산이 밀집해 있습니다.",
                37.5658,
                126.9752,
                16.0,
                "산책",
                r#"["산책","돌담길","데이트코스","가을단풍","문화유산"]"#,
                "길따라발길따라",
                24i64,
            ),
            (
                "성수동 카페거리 (Seongsu Cafe Street)",
                "seongsu-cafe-street",
                "옛 공장과 붉은 벽돌 창고가 트렌디한 갤러리와 카페로 재탄생한 복합문화 구역.",
                "# 성수동 카페거리 (서울의 브루클린)\n\n과거 수제화 공장과 인쇄소, 금속 공업사들이 밀집했던 준공업지역이 독창적인 F&B, 팝업스토어, 패션 브랜드의 성지가 되었습니다.\n\n## 둘러볼 곳\n- **대림창고**: 공장의 거친 골조를 그대로 살린 복합문화 갤러리 카페\n- **성수 연방**: 개성 넘치는 편집숍과 라이프스타일 샵",
                37.5445,
                127.0560,
                16.0,
                "카페",
                r#"["카페","팝업스토어","성수","핫플레이스","디자인"]"#,
                "트렌드헌터",
                78i64,
            ),
        ];

        for (idx, (title, slug, summary, content, lat, lng, zoom, cat, tags, author, views)) in articles.iter().enumerate() {
            let art_id = Uuid::new_v4();
            let rev_id = Uuid::new_v4();

            let _ = db_execute!(
                self,
                "INSERT INTO wiki_articles (id, title, slug, summary, content, latitude, longitude, zoom, category, tags, author, views, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)",
                art_id,
                title,
                slug,
                summary,
                content,
                lat,
                lng,
                zoom,
                cat,
                tags,
                author,
                views,
                now
            );

            let _ = db_execute!(
                self,
                "INSERT INTO wiki_revisions (id, article_id, title, content, latitude, longitude, edit_summary, author, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, '초기 문서 생성', $7, $8)",
                rev_id,
                art_id,
                title,
                content,
                lat,
                lng,
                author,
                now
            );

            let log_count = (10 - idx * 2).max(2);
            for j in 0..log_count {
                let log_id = Uuid::new_v4();
                let minutes_ago = (j as i64 * 15) + (idx as i64 * 20);
                let log_time = now - Duration::minutes(minutes_ago);
                let _ = db_execute!(
                    self,
                    "INSERT INTO wiki_view_logs (id, article_id, created_at) VALUES ($1, $2, $3)",
                    log_id,
                    art_id,
                    log_time
                );
            }
        }

        info!(target: "serve::db", "Wiki test data seeded successfully!");
        Ok(())
    }
}
