use crate::domain::models::{Comment, Poll, PollOption, Post, PostMedia, User, WikiRevision};
use crate::infrastructure::db::entities::{
    CommentEntity, PollEntity, PollOptionEntity, PostEntity, PostMediaEntity, UserEntity,
    WikiRevisionEntity,
};
use crate::infrastructure::db::postgres::{Database, DatabaseBackend};

use async_graphql::dataloader::Loader;
use chrono::Utc;
use sqlx::{AssertSqlSafe, Row};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn sqlite_in(base: &str, field: &str, count: usize, suffix: &str) -> String {
    let placeholders = vec!["?"; count].join(",");
    format!("{base} WHERE {field} IN ({placeholders}) {suffix}")
}

/// Batch loads User entities by their IDs to eliminate N+1 queries
pub struct UserLoader {
    db: Database,
}

impl UserLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for UserLoader {
    type Value = User;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let entities: Vec<UserEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE id = ANY($1)")
                    .bind(keys)
                    .fetch_all(pool)
                    .await
                    .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in("SELECT * FROM users", "id", keys.len(), "");
                let mut query = sqlx::query_as::<_, UserEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        Ok(entities
            .into_iter()
            .map(|e| {
                let u = User::from(e);
                (u.id, u)
            })
            .collect())
    }
}

/// Batch loads post likes count
pub struct PostLikesCountLoader {
    db: Database,
}

impl PostLikesCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PostLikesCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT post_id, COUNT(*) as cnt FROM likes WHERE post_id = ANY($1) GROUP BY post_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let pid: Uuid = row.get("post_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(pid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT post_id, COUNT(*) as cnt FROM likes",
                    "post_id",
                    keys.len(),
                    "GROUP BY post_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let pid: Uuid = row.get("post_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(pid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads comment likes count
pub struct CommentLikesCountLoader {
    db: Database,
}

impl CommentLikesCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for CommentLikesCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT comment_id, COUNT(*) as cnt FROM comment_likes WHERE comment_id = ANY($1) GROUP BY comment_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let cid: Uuid = row.get("comment_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(cid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT comment_id, COUNT(*) as cnt FROM comment_likes",
                    "comment_id",
                    keys.len(),
                    "GROUP BY comment_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let cid: Uuid = row.get("comment_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(cid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads post reposts count
pub struct PostRepostsCountLoader {
    db: Database,
}

impl PostRepostsCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PostRepostsCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT post_id, COUNT(*) as cnt FROM reposts WHERE post_id = ANY($1) GROUP BY post_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let pid: Uuid = row.get("post_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(pid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT post_id, COUNT(*) as cnt FROM reposts",
                    "post_id",
                    keys.len(),
                    "GROUP BY post_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let pid: Uuid = row.get("post_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(pid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads user posts count
pub struct UserPostsCountLoader {
    db: Database,
}

impl UserPostsCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for UserPostsCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT author_id, COUNT(*) as cnt FROM posts WHERE author_id = ANY($1) GROUP BY author_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let uid: Uuid = row.get("author_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT author_id, COUNT(*) as cnt FROM posts",
                    "author_id",
                    keys.len(),
                    "GROUP BY author_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let uid: Uuid = row.get("author_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads followers count for users
pub struct FollowersCountLoader {
    db: Database,
}

impl FollowersCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for FollowersCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT followee_id, COUNT(*) as cnt FROM follows WHERE followee_id = ANY($1) GROUP BY followee_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let uid: Uuid = row.get("followee_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT followee_id, COUNT(*) as cnt FROM follows",
                    "followee_id",
                    keys.len(),
                    "GROUP BY followee_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let uid: Uuid = row.get("followee_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads following count for users
pub struct FollowingCountLoader {
    db: Database,
}

impl FollowingCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for FollowingCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT follower_id, COUNT(*) as cnt FROM follows WHERE follower_id = ANY($1) GROUP BY follower_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let uid: Uuid = row.get("follower_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT follower_id, COUNT(*) as cnt FROM follows",
                    "follower_id",
                    keys.len(),
                    "GROUP BY follower_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let uid: Uuid = row.get("follower_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(uid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads whether users have active stories in the last 24h
pub struct HasActiveStoriesLoader {
    db: Database,
}

impl HasActiveStoriesLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for HasActiveStoriesLoader {
    type Value = bool;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, false);
        }

        let now = Utc::now();
        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT DISTINCT author_id FROM stories WHERE author_id = ANY($1) AND expires_at > $2",
                )
                .bind(keys)
                .bind(now)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let uid: Uuid = row.get("author_id");
                    map.insert(uid, true);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT DISTINCT author_id FROM stories",
                    "author_id",
                    keys.len(),
                    "AND expires_at > ?",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query = query.bind(now);
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let uid: Uuid = row.get("author_id");
                    map.insert(uid, true);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads comment replies count
pub struct CommentRepliesCountLoader {
    db: Database,
}

impl CommentRepliesCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for CommentRepliesCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT parent_id, COUNT(*) as cnt FROM comments WHERE parent_id = ANY($1) GROUP BY parent_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    if let Some(pid) = row.get::<Option<Uuid>, _>("parent_id") {
                        let cnt: i64 = row.get("cnt");
                        map.insert(pid, cnt as usize);
                    }
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT parent_id, COUNT(*) as cnt FROM comments",
                    "parent_id",
                    keys.len(),
                    "GROUP BY parent_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    if let Some(pid) = row.get::<Option<Uuid>, _>("parent_id") {
                        let cnt: i64 = row.get("cnt");
                        map.insert(pid, cnt as usize);
                    }
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads post media attachments
pub struct PostMediaLoader {
    db: Database,
}

impl PostMediaLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PostMediaLoader {
    type Value = Vec<PostMedia>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, Vec::new());
        }

        let entities: Vec<PostMediaEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, PostMediaEntity>(
                    "SELECT * FROM post_media WHERE post_id = ANY($1) ORDER BY sort_order ASC, created_at ASC",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in("SELECT * FROM post_media", "post_id", keys.len(), "ORDER BY sort_order ASC, created_at ASC");
                let mut query = sqlx::query_as::<_, PostMediaEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        for e in entities {
            let m = PostMedia::from(e);
            map.entry(m.post_id).or_default().push(m);
        }
        Ok(map)
    }
}

/// Batch loads poll for posts
pub struct PostPollLoader {
    db: Database,
}

impl PostPollLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PostPollLoader {
    type Value = Option<Poll>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, None);
        }

        let entities: Vec<PollEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, PollEntity>("SELECT * FROM polls WHERE post_id = ANY($1)")
                    .bind(keys)
                    .fetch_all(pool)
                    .await
                    .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in("SELECT * FROM polls", "post_id", keys.len(), "");
                let mut query = sqlx::query_as::<_, PollEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        for e in entities {
            let poll = Poll::from(e);
            map.insert(poll.post_id, Some(poll));
        }
        Ok(map)
    }
}

/// Batch loads poll options for polls
pub struct PollOptionsLoader {
    db: Database,
}

impl PollOptionsLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PollOptionsLoader {
    type Value = Vec<PollOption>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, Vec::new());
        }

        let entities: Vec<PollOptionEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => sqlx::query_as::<_, PollOptionEntity>(
                "SELECT * FROM poll_options WHERE poll_id = ANY($1) ORDER BY sort_order ASC",
            )
            .bind(keys)
            .fetch_all(pool)
            .await
            .map_err(Arc::new)?,
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT * FROM poll_options",
                    "poll_id",
                    keys.len(),
                    "ORDER BY sort_order ASC",
                );
                let mut query = sqlx::query_as::<_, PollOptionEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        for e in entities {
            let opt = PollOption::from(e);
            map.entry(opt.poll_id).or_default().push(opt);
        }
        Ok(map)
    }
}

/// Batch loads poll option votes count
pub struct PollOptionVotesCountLoader {
    db: Database,
}

impl PollOptionVotesCountLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PollOptionVotesCountLoader {
    type Value = usize;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        for key in keys {
            map.insert(*key, 0);
        }

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT option_id, COUNT(*) as cnt FROM poll_votes WHERE option_id = ANY($1) GROUP BY option_id",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let oid: Uuid = row.get("option_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(oid, cnt as usize);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT option_id, COUNT(*) as cnt FROM poll_votes",
                    "option_id",
                    keys.len(),
                    "GROUP BY option_id",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let oid: Uuid = row.get("option_id");
                    let cnt: i64 = row.get("cnt");
                    map.insert(oid, cnt as usize);
                }
            }
        }
        Ok(map)
    }
}

/// Batch loads Post entities by their IDs to eliminate N+1 queries
pub struct PostLoader {
    db: Database,
}

impl PostLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for PostLoader {
    type Value = Post;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let entities: Vec<PostEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, PostEntity>("SELECT * FROM posts WHERE id = ANY($1)")
                    .bind(keys)
                    .fetch_all(pool)
                    .await
                    .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in("SELECT * FROM posts", "id", keys.len(), "");
                let mut query = sqlx::query_as::<_, PostEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        Ok(entities
            .into_iter()
            .map(|e| {
                let p = Post::from(e);
                (p.id, p)
            })
            .collect())
    }
}

/// Batch loads Comment entities by their IDs to eliminate N+1 queries
pub struct CommentLoader {
    db: Database,
}

impl CommentLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for CommentLoader {
    type Value = Comment;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let entities: Vec<CommentEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, CommentEntity>("SELECT * FROM comments WHERE id = ANY($1)")
                    .bind(keys)
                    .fetch_all(pool)
                    .await
                    .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in("SELECT * FROM comments", "id", keys.len(), "");
                let mut query = sqlx::query_as::<_, CommentEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        Ok(entities
            .into_iter()
            .map(|entity| {
                let c = Comment::from(entity);
                (c.id, c)
            })
            .collect())
    }
}

/// Batch loads WikiRevision entities by their article IDs
pub struct WikiRevisionsLoader {
    db: Database,
}

impl WikiRevisionsLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<Uuid> for WikiRevisionsLoader {
    type Value = Vec<WikiRevision>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let entities: Vec<WikiRevisionEntity> = match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                sqlx::query_as::<_, WikiRevisionEntity>(
                    "SELECT * FROM wiki_revisions WHERE article_id = ANY($1) ORDER BY created_at DESC",
                )
                .bind(keys)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?
            }
            DatabaseBackend::Sqlite(pool) => {
                let sql = sqlite_in(
                    "SELECT * FROM wiki_revisions",
                    "article_id",
                    keys.len(),
                    "ORDER BY created_at DESC",
                );
                let mut query = sqlx::query_as::<_, WikiRevisionEntity>(AssertSqlSafe(sql.as_str()));
                for k in keys {
                    query = query.bind(k);
                }
                query.fetch_all(pool).await.map_err(Arc::new)?
            }
        };

        let mut map: HashMap<Uuid, Vec<WikiRevision>> = HashMap::new();
        for k in keys {
            map.insert(*k, Vec::new());
        }
        for entity in entities {
            let rev = WikiRevision::from(entity);
            map.entry(rev.article_id).or_default().push(rev);
        }

        Ok(map)
    }
}

/// Batch loads follow relationships (follower_id, followee_id) -> bool
pub struct IsFollowingLoader {
    db: Database,
}

impl IsFollowingLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<(Uuid, Uuid)> for IsFollowingLoader {
    type Value = bool;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map: HashMap<(Uuid, Uuid), bool> = keys.iter().map(|k| (*k, false)).collect();

        let mut followers: Vec<Uuid> = keys.iter().map(|(f, _)| *f).collect();
        followers.sort_unstable();
        followers.dedup();

        let mut followees: Vec<Uuid> = keys.iter().map(|(_, f)| *f).collect();
        followees.sort_unstable();
        followees.dedup();

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT follower_id, followee_id FROM follows WHERE follower_id = ANY($1) AND followee_id = ANY($2)",
                )
                .bind(&followers)
                .bind(&followees)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let f: Uuid = row.try_get("follower_id").map_err(Arc::new)?;
                    let fe: Uuid = row.try_get("followee_id").map_err(Arc::new)?;
                    map.insert((f, fe), true);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let p1 = vec!["?"; followers.len()].join(",");
                let p2 = vec!["?"; followees.len()].join(",");
                let sql = format!(
                    "SELECT follower_id, followee_id FROM follows WHERE follower_id IN ({p1}) AND followee_id IN ({p2})",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for f in &followers {
                    query = query.bind(f);
                }
                for fe in &followees {
                    query = query.bind(fe);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let f: Uuid = row.try_get("follower_id").map_err(Arc::new)?;
                    let fe: Uuid = row.try_get("followee_id").map_err(Arc::new)?;
                    map.insert((f, fe), true);
                }
            }
        }

        Ok(map)
    }
}

/// Batch loads block relationships (blocker_id, blocked_id) -> bool
pub struct IsBlockedLoader {
    db: Database,
}

impl IsBlockedLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<(Uuid, Uuid)> for IsBlockedLoader {
    type Value = bool;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map: HashMap<(Uuid, Uuid), bool> = keys.iter().map(|k| (*k, false)).collect();

        let mut blockers: Vec<Uuid> = keys.iter().map(|(b, _)| *b).collect();
        blockers.sort_unstable();
        blockers.dedup();

        let mut blockeds: Vec<Uuid> = keys.iter().map(|(_, b)| *b).collect();
        blockeds.sort_unstable();
        blockeds.dedup();

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT blocker_id, blocked_id FROM blocks WHERE blocker_id = ANY($1) AND blocked_id = ANY($2)",
                )
                .bind(&blockers)
                .bind(&blockeds)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let b: Uuid = row.try_get("blocker_id").map_err(Arc::new)?;
                    let bd: Uuid = row.try_get("blocked_id").map_err(Arc::new)?;
                    map.insert((b, bd), true);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let p1 = vec!["?"; blockers.len()].join(",");
                let p2 = vec!["?"; blockeds.len()].join(",");
                let sql = format!(
                    "SELECT blocker_id, blocked_id FROM blocks WHERE blocker_id IN ({p1}) AND blocked_id IN ({p2})",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for b in &blockers {
                    query = query.bind(b);
                }
                for bd in &blockeds {
                    query = query.bind(bd);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let b: Uuid = row.try_get("blocker_id").map_err(Arc::new)?;
                    let bd: Uuid = row.try_get("blocked_id").map_err(Arc::new)?;
                    map.insert((b, bd), true);
                }
            }
        }

        Ok(map)
    }
}

/// Batch loads mute relationships (muter_id, muted_id) -> bool
pub struct IsMutedLoader {
    db: Database,
}

impl IsMutedLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<(Uuid, Uuid)> for IsMutedLoader {
    type Value = bool;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map: HashMap<(Uuid, Uuid), bool> = keys.iter().map(|k| (*k, false)).collect();

        let mut muters: Vec<Uuid> = keys.iter().map(|(m, _)| *m).collect();
        muters.sort_unstable();
        muters.dedup();

        let mut muteds: Vec<Uuid> = keys.iter().map(|(_, m)| *m).collect();
        muteds.sort_unstable();
        muteds.dedup();

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT muter_id, muted_id FROM mutes WHERE muter_id = ANY($1) AND muted_id = ANY($2)",
                )
                .bind(&muters)
                .bind(&muteds)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let m: Uuid = row.try_get("muter_id").map_err(Arc::new)?;
                    let md: Uuid = row.try_get("muted_id").map_err(Arc::new)?;
                    map.insert((m, md), true);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let p1 = vec!["?"; muters.len()].join(",");
                let p2 = vec!["?"; muteds.len()].join(",");
                let sql = format!(
                    "SELECT muter_id, muted_id FROM mutes WHERE muter_id IN ({p1}) AND muted_id IN ({p2})",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for m in &muters {
                    query = query.bind(m);
                }
                for md in &muteds {
                    query = query.bind(md);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let m: Uuid = row.try_get("muter_id").map_err(Arc::new)?;
                    let md: Uuid = row.try_get("muted_id").map_err(Arc::new)?;
                    map.insert((m, md), true);
                }
            }
        }

        Ok(map)
    }
}

/// Batch loads pending follow requests (requester_id, target_id) -> bool
pub struct HasPendingFollowRequestLoader {
    db: Database,
}

impl HasPendingFollowRequestLoader {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl Loader<(Uuid, Uuid)> for HasPendingFollowRequestLoader {
    type Value = bool;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[(Uuid, Uuid)],
    ) -> Result<HashMap<(Uuid, Uuid), Self::Value>, Self::Error> {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        let mut map: HashMap<(Uuid, Uuid), bool> = keys.iter().map(|k| (*k, false)).collect();

        let mut requesters: Vec<Uuid> = keys.iter().map(|(r, _)| *r).collect();
        requesters.sort_unstable();
        requesters.dedup();

        let mut targets: Vec<Uuid> = keys.iter().map(|(_, t)| *t).collect();
        targets.sort_unstable();
        targets.dedup();

        match self.db.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT requester_id, target_id FROM follow_requests WHERE requester_id = ANY($1) AND target_id = ANY($2) AND status = 'PENDING'",
                )
                .bind(&requesters)
                .bind(&targets)
                .fetch_all(pool)
                .await
                .map_err(Arc::new)?;

                for row in rows {
                    let r: Uuid = row.try_get("requester_id").map_err(Arc::new)?;
                    let t: Uuid = row.try_get("target_id").map_err(Arc::new)?;
                    map.insert((r, t), true);
                }
            }
            DatabaseBackend::Sqlite(pool) => {
                let p1 = vec!["?"; requesters.len()].join(",");
                let p2 = vec!["?"; targets.len()].join(",");
                let sql = format!(
                    "SELECT requester_id, target_id FROM follow_requests WHERE requester_id IN ({p1}) AND target_id IN ({p2}) AND status = 'PENDING'",
                );
                let mut query = sqlx::query(AssertSqlSafe(sql.as_str()));
                for r in &requesters {
                    query = query.bind(r);
                }
                for t in &targets {
                    query = query.bind(t);
                }
                let rows = query.fetch_all(pool).await.map_err(Arc::new)?;
                for row in rows {
                    let r: Uuid = row.try_get("requester_id").map_err(Arc::new)?;
                    let t: Uuid = row.try_get("target_id").map_err(Arc::new)?;
                    map.insert((r, t), true);
                }
            }
        }

        Ok(map)
    }
}


