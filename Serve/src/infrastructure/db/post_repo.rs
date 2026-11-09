use super::database::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    Comment, Post, PostAnalytics, PostAudience, PostMedia, User, UserList,
};
use crate::domain::repositories::PostRepository;
use crate::infrastructure::db::entities::{
    CommentEntity, HashtagEntity, PostEntity, PostMediaEntity, UserEntity, UserListEntity,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

impl Database {
    async fn index_post_content(&self, post_id: Uuid, content: &str) {
        let now = Utc::now();
        // 1. Extract hashtags
        for word in content.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                let clean_tag: String = tag
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !clean_tag.is_empty() {
                    let tag_lower = clean_tag.to_lowercase();
                    let id = Uuid::new_v4();
                    let _ = db_execute!(
                        self,
                        "INSERT INTO hashtags (id, name, posts_count, created_at)
                         VALUES ($1, $2, 1, $3)
                         ON CONFLICT (name) DO UPDATE SET posts_count = hashtags.posts_count + 1",
                        id,
                        &tag_lower,
                        now
                    );

                    let _ = db_execute!(
                        self,
                        "INSERT INTO post_hashtags (post_id, hashtag_id, created_at)
                         SELECT $1, id, $2 FROM hashtags WHERE name = $3
                         ON CONFLICT DO NOTHING",
                        post_id,
                        now,
                        &tag_lower
                    );
                }
            }
        }

        // 2. Extract mentions
        for word in content.split_whitespace() {
            if let Some(handle) = word.strip_prefix('@') {
                let clean_handle: String = handle
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !clean_handle.is_empty() {
                    let handle_lower = clean_handle.to_lowercase();
                    let _ = db_execute!(
                        self,
                        "INSERT INTO user_mentions (post_id, user_id, created_at)
                         SELECT $1, id, $2 FROM users WHERE LOWER(username) = $3
                         ON CONFLICT DO NOTHING",
                        post_id,
                        now,
                        &handle_lower
                    );
                }
            }
        }
    }
}

#[async_trait]
impl PostRepository for Database {
    async fn get_user_posts_count(&self, user_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM posts WHERE author_id = $1",
            user_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_liked_posts_by_user(&self, user_id: Uuid) -> Result<Vec<Post>, DomainError> {
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT p.* FROM posts p JOIN likes l ON p.id = l.post_id WHERE l.user_id = $1 ORDER BY l.created_at DESC",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, user_id = %user_id, "Failed to get liked posts");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get liked posts")
        })?;

        Ok(entities.into_iter().map(Post::from).collect())
    }

    async fn get_comments_by_user(&self, user_id: Uuid) -> Result<Vec<Comment>, DomainError> {
        let entities: Vec<CommentEntity> = db_fetch_all!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE author_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, user_id = %user_id, "Failed to get comments by user");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get comments by user")
        })?;

        Ok(entities.into_iter().map(Comment::from).collect())
    }

    async fn get_posts(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
            lim,
            off
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, "Failed to get posts");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get posts")
        })?;

        Ok(entities.into_iter().map(Post::from).collect())
    }

    async fn get_posts_cursor(
        &self,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError> {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT * FROM posts
                 WHERE ((created_at < $1) OR (created_at = $1 AND id < $2))
                 ORDER BY created_at DESC, id DESC
                 LIMIT $3",
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT * FROM posts
                 ORDER BY created_at DESC, id DESC
                 LIMIT $1",
                fetch_limit
            ),
        }
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, "Failed to get posts cursor");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get posts cursor")
        })?;

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        Ok((items, has_next_page))
    }

    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError> {
        let entity: Option<PostEntity> = db_fetch_optional!(
            self,
            PostEntity,
            "SELECT * FROM posts WHERE id = $1",
            id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, id = %id, "Failed to get post by id");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get post by id")
        })?;

        Ok(entity.map(Post::from))
    }

    async fn get_posts_by_author(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError> {
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts WHERE author_id = $1 ORDER BY created_at DESC, id DESC",
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, author_id = %author_id, "Failed to get posts by author");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get posts by author")
        })?;

        Ok(entities.into_iter().map(Post::from).collect())
    }

    async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError> {
        self.create_post_with_details(author_id, content, audience, Vec::new(), None)
            .await
    }

    async fn create_post_with_details(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
        mut media: Vec<PostMedia>,
        poll: Option<(String, Vec<String>, i64)>,
    ) -> Result<Post, DomainError> {
        let post_id = Uuid::new_v4();
        let aud = audience.unwrap_or(PostAudience::Public);
        let now = Utc::now();

        for m in &mut media {
            m.post_id = post_id;
        }

        let mut tx = self.begin().await.map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to begin transaction");
            DomainError::new(ErrorCode::PostCreateFailed, "Failed to begin transaction")
        })?;

        // 1. Insert post
        db_tx_execute!(
            &mut tx,
            "INSERT INTO posts (id, author_id, content, quote_post_id, pinned_comment_id, audience, views_count, created_at)
             VALUES ($1, $2, $3, NULL, NULL, $4, 0, $5)",
            post_id,
            author_id,
            &content,
            aud.as_str(),
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create post");
            DomainError::new(ErrorCode::PostCreateFailed, "Failed to create post")
        })?;

        // 2. Insert media attachments
        for m in &media {
            db_tx_execute!(
                &mut tx,
                "INSERT INTO post_media (id, post_id, media_url, media_type, alt_text, sort_order, width, height, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                m.id,
                m.post_id,
                &m.media_url,
                m.media_type.as_str(),
                &m.alt_text,
                m.sort_order,
                m.width,
                m.height,
                m.created_at
            )
            .map_err(|e| {
                error!(target: "serve::db", error = %e, "Failed to insert post media");
                DomainError::new(ErrorCode::PostCreateFailed, "Failed to save post media")
            })?;
        }

        // 3. Insert poll if present
        if let Some((question, options, duration_secs)) = poll {
            let poll_id = Uuid::new_v4();
            let expires_at = now + chrono::Duration::seconds(duration_secs);

            db_tx_execute!(
                &mut tx,
                "INSERT INTO polls (id, post_id, question, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)",
                poll_id,
                post_id,
                &question,
                expires_at,
                now
            )
            .map_err(|e| {
                error!(target: "serve::db", error = %e, "Failed to create poll");
                DomainError::new(ErrorCode::PollInvalidOptions, "Failed to create poll")
            })?;

            for (idx, opt_text) in options.iter().enumerate() {
                let opt_id = Uuid::new_v4();
                db_tx_execute!(
                    &mut tx,
                    "INSERT INTO poll_options (id, poll_id, option_text, sort_order) VALUES ($1, $2, $3, $4)",
                    opt_id,
                    poll_id,
                    opt_text,
                    idx as i32
                )
                .map_err(|e| {
                    error!(target: "serve::db", error = %e, "Failed to insert poll option");
                    DomainError::new(ErrorCode::PollInvalidOptions, "Failed to create poll option")
                })?;
            }
        }

        // 4. Index hashtags
        for word in content.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                let clean_tag: String = tag
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !clean_tag.is_empty() {
                    let tag_lower = clean_tag.to_lowercase();
                    let h_id = Uuid::new_v4();
                    db_tx_execute!(
                        &mut tx,
                        "INSERT INTO hashtags (id, name, posts_count, created_at)
                         VALUES ($1, $2, 1, $3)
                         ON CONFLICT (name) DO UPDATE SET posts_count = hashtags.posts_count + 1",
                        h_id,
                        &tag_lower,
                        now
                    )
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Failed to index hashtag");
                        DomainError::new(ErrorCode::PostCreateFailed, "Failed to index hashtag")
                    })?;

                    db_tx_execute!(
                        &mut tx,
                        "INSERT INTO post_hashtags (post_id, hashtag_id, created_at)
                         SELECT $1, id, $2 FROM hashtags WHERE name = $3
                         ON CONFLICT DO NOTHING",
                        post_id,
                        now,
                        &tag_lower
                    )
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Failed to link post hashtag");
                        DomainError::new(ErrorCode::PostCreateFailed, "Failed to link post hashtag")
                    })?;
                }
            }
        }

        // 5. Index user mentions
        for word in content.split_whitespace() {
            if let Some(handle) = word.strip_prefix('@') {
                let clean_handle: String = handle
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !clean_handle.is_empty() {
                    let handle_lower = clean_handle.to_lowercase();
                    db_tx_execute!(
                        &mut tx,
                        "INSERT INTO user_mentions (post_id, user_id, created_at)
                         SELECT $1, id, $2 FROM users WHERE LOWER(username) = $3
                         ON CONFLICT DO NOTHING",
                        post_id,
                        now,
                        &handle_lower
                    )
                    .map_err(|e| {
                        error!(target: "serve::db", error = %e, "Failed to index user mention");
                        DomainError::new(ErrorCode::PostCreateFailed, "Failed to index mention")
                    })?;
                }
            }
        }

        tx.commit().await.map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to commit post creation transaction");
            DomainError::new(ErrorCode::PostCreateFailed, "Failed to commit transaction")
        })?;

        Ok(Post {
            id: post_id,
            author_id,
            content,
            quote_post_id: None,
            pinned_comment_id: None,
            audience: aud,
            views_count: 0,
            created_at: now,
        })
    }

    async fn create_quote_post(
        &self,
        author_id: Uuid,
        quote_post_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        let _ = self
            .get_post_by_id(quote_post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Target post not found"))?;

        let id = Uuid::new_v4();
        let aud = PostAudience::Public;
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO posts (id, author_id, content, quote_post_id, pinned_comment_id, audience, views_count, created_at)
             VALUES ($1, $2, $3, $4, NULL, $5, 0, $6)",
            id,
            author_id,
            &content,
            quote_post_id,
            aud.as_str(),
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create quote post");
            DomainError::new(ErrorCode::PostCreateFailed, "Failed to create quote post")
        })?;

        self.index_post_content(id, &content).await;

        Ok(Post {
            id,
            author_id,
            content,
            quote_post_id: Some(quote_post_id),
            pinned_comment_id: None,
            audience: aud,
            views_count: 0,
            created_at: now,
        })
    }

    async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        let res = db_execute!(
            self,
            "UPDATE posts SET content = $1 WHERE id = $2 AND author_id = $3",
            &content,
            post_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update post");
            DomainError::new(ErrorCode::PostUpdateFailed, "Failed to update post")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::PostNotFound,
                "Post not found or unauthorized",
            ));
        }

        self.get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))
    }

    async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        let result = db_execute!(
            self,
            "DELETE FROM posts WHERE id = $1 AND author_id = $2",
            post_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete post");
            DomainError::new(ErrorCode::PostDeleteFailed, "Failed to delete post")
        })?;

        if result.rows_affected() > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::PostNotFound,
                "Post not found or unauthorized",
            ))
        }
    }

    async fn get_feed(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT p.* FROM posts p
             WHERE (
                 p.author_id = $1
                 OR p.author_id IN (SELECT followee_id FROM follows WHERE follower_id = $1)
             )
             AND p.author_id NOT IN (
                 SELECT blocked_id FROM blocks WHERE blocker_id = $1
                 UNION
                 SELECT blocker_id FROM blocks WHERE blocked_id = $1
             )
             AND p.author_id NOT IN (
                 SELECT muted_id FROM mutes WHERE muter_id = $1
             )
             ORDER BY p.created_at DESC, p.id DESC
             LIMIT $2 OFFSET $3",
            user_id,
            lim,
            off
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, user_id = %user_id, "Failed to get feed");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get feed")
        })?;

        Ok(entities.into_iter().map(Post::from).collect())
    }

    async fn get_feed_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError> {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 WHERE (
                     p.author_id = $1
                     OR p.author_id IN (SELECT followee_id FROM follows WHERE follower_id = $1)
                 )
                 AND p.author_id NOT IN (
                     SELECT blocked_id FROM blocks WHERE blocker_id = $1
                     UNION
                     SELECT blocker_id FROM blocks WHERE blocked_id = $1
                 )
                 AND p.author_id NOT IN (
                     SELECT muted_id FROM mutes WHERE muter_id = $1
                 )
                 AND ((p.created_at < $2) OR (p.created_at = $2 AND p.id < $3))
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $4",
                user_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 WHERE (
                     p.author_id = $1
                     OR p.author_id IN (SELECT followee_id FROM follows WHERE follower_id = $1)
                 )
                 AND p.author_id NOT IN (
                     SELECT blocked_id FROM blocks WHERE blocker_id = $1
                     UNION
                     SELECT blocker_id FROM blocks WHERE blocked_id = $1
                 )
                 AND p.author_id NOT IN (
                     SELECT muted_id FROM mutes WHERE muter_id = $1
                 )
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $2",
                user_id,
                fetch_limit
            ),
        }
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, user_id = %user_id, "Failed to get feed cursor");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get feed cursor")
        })?;

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        Ok((items, has_next_page))
    }

    async fn search_posts(&self, query: &str) -> Result<Vec<Post>, DomainError> {
        let search_pattern = format!("%{}%", query);
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts WHERE content LIKE $1 ORDER BY created_at DESC, id DESC LIMIT 50",
            &search_pattern
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, query = %query, "Failed to search posts");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to search posts")
        })?;

        Ok(entities.into_iter().map(Post::from).collect())
    }

    async fn search_posts_cursor(
        &self,
        query: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError> {
        let fetch_limit = (first + 1) as i64;
        let search_pattern = format!("%{}%", query);
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT * FROM posts
                 WHERE content LIKE $1
                   AND ((created_at < $2) OR (created_at = $2 AND id < $3))
                 ORDER BY created_at DESC, id DESC
                 LIMIT $4",
                &search_pattern,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT * FROM posts
                 WHERE content LIKE $1
                 ORDER BY created_at DESC, id DESC
                 LIMIT $2",
                &search_pattern,
                fetch_limit
            ),
        }
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, query = %query, "Failed to search posts cursor");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to search posts cursor")
        })?;

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        Ok((items, has_next_page))
    }

    async fn get_posts_by_hashtag_cursor(
        &self,
        hashtag: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError> {
        let fetch_limit = (first + 1) as i64;
        let tag_lower = hashtag.trim_start_matches('#').to_lowercase();

        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN post_hashtags ph ON p.id = ph.post_id
                 JOIN hashtags h ON ph.hashtag_id = h.id
                 WHERE h.name = $1
                   AND ((p.created_at < $2) OR (p.created_at = $2 AND p.id < $3))
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $4",
                &tag_lower,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN post_hashtags ph ON p.id = ph.post_id
                 JOIN hashtags h ON ph.hashtag_id = h.id
                 WHERE h.name = $1
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $2",
                &tag_lower,
                fetch_limit
            ),
        }
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, hashtag = %hashtag, "Failed to get posts by hashtag");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get posts by hashtag")
        })?;

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        Ok((items, has_next_page))
    }

    // Likes & Reposts
    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "INSERT INTO likes (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            post_id,
            Utc::now()
        );

        Ok(post)
    }

    async fn unlike_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "DELETE FROM likes WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        );

        Ok(post)
    }

    async fn get_likes_count(&self, post_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM likes WHERE post_id = $1",
            post_id
        )
        .unwrap_or(0) as usize
    }

    async fn is_post_liked_by(&self, post_id: Uuid, user_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND post_id = $2)",
            user_id,
            post_id
        )
        .unwrap_or(false)
    }

    async fn repost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "INSERT INTO reposts (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            post_id,
            Utc::now()
        );

        Ok(post)
    }

    async fn unrepost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "DELETE FROM reposts WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        );

        Ok(post)
    }

    async fn get_reposts_count(&self, post_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM reposts WHERE post_id = $1",
            post_id
        )
        .unwrap_or(0) as usize
    }

    async fn is_post_reposted_by(&self, post_id: Uuid, user_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM reposts WHERE user_id = $1 AND post_id = $2)",
            user_id,
            post_id
        )
        .unwrap_or(false)
    }

    async fn get_trending_hashtags(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<(String, usize)>, DomainError> {
        let lim = limit.unwrap_or(10) as i64;
        let entities: Vec<HashtagEntity> = db_fetch_all!(
            self,
            HashtagEntity,
            "SELECT * FROM hashtags ORDER BY posts_count DESC LIMIT $1",
            lim
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, "Failed to get trending hashtags");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get trending hashtags")
        })?;

        Ok(entities
            .into_iter()
            .map(|h| (h.name, h.posts_count as usize))
            .collect())
    }

    // 1. Media Attachments
    async fn add_post_media(
        &self,
        post_id: Uuid,
        media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError> {
        let _ = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        for m in &media {
            db_execute!(
                self,
                "INSERT INTO post_media (id, post_id, media_url, media_type, alt_text, sort_order, width, height, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                m.id,
                m.post_id,
                &m.media_url,
                m.media_type.as_str(),
                &m.alt_text,
                m.sort_order,
                m.width,
                m.height,
                m.created_at
            )
            .map_err(|e| {
                error!(target: "serve::db", error = %e, "Failed to insert post media");
                DomainError::new(ErrorCode::PostCreateFailed, "Failed to save post media")
            })?;
        }

        Ok(media)
    }

    async fn get_post_media(&self, post_id: Uuid) -> Result<Vec<PostMedia>, DomainError> {
        let entities: Vec<PostMediaEntity> = db_fetch_all!(
            self,
            PostMediaEntity,
            "SELECT * FROM post_media WHERE post_id = $1 ORDER BY sort_order ASC, created_at ASC",
            post_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, post_id = %post_id, "Failed to get post media");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get post media")
        })?;

        Ok(entities.into_iter().map(PostMedia::from).collect())
    }

    // 2. User Lists & Custom Feeds
    async fn create_user_list(
        &self,
        owner_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
        member_ids: Vec<Uuid>,
    ) -> Result<UserList, DomainError> {
        let list_id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO user_lists (id, owner_id, name, description, is_private, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            list_id,
            owner_id,
            &name,
            &description,
            is_private,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create user list");
            DomainError::new(ErrorCode::ListNotFound, "Failed to create list")
        })?;

        for mid in member_ids {
            let _ = db_execute!(
                self,
                "INSERT INTO user_list_members (list_id, user_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                list_id,
                mid,
                now
            );
        }

        Ok(UserList {
            id: list_id,
            owner_id,
            name,
            description,
            is_private,
            created_at: now,
        })
    }

    async fn update_user_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<UserList, DomainError> {
        let current = self
            .get_user_list_by_id(list_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::ListNotFound, "User list not found"))?;

        if current.owner_id != owner_id {
            return Err(DomainError::new(
                ErrorCode::ListUnauthorized,
                "Only owner can edit list",
            ));
        }

        let new_name = name.unwrap_or(current.name);
        let new_desc = description.or(current.description);
        let new_private = is_private.unwrap_or(current.is_private);

        let res = db_execute!(
            self,
            "UPDATE user_lists SET name = $1, description = $2, is_private = $3 WHERE id = $4 AND owner_id = $5",
            &new_name,
            &new_desc,
            new_private,
            list_id,
            owner_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update list");
            DomainError::new(ErrorCode::ListNotFound, "Failed to update list")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::ListNotFound,
                "User list not found",
            ));
        }

        self.get_user_list_by_id(list_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::ListNotFound, "User list not found"))
    }

    async fn delete_user_list(&self, list_id: Uuid, owner_id: Uuid) -> Result<bool, DomainError> {
        let result = db_execute!(
            self,
            "DELETE FROM user_lists WHERE id = $1 AND owner_id = $2",
            list_id,
            owner_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete list");
            DomainError::new(ErrorCode::ListNotFound, "Failed to delete list")
        })?;

        if result.rows_affected() > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::ListNotFound,
                "List not found or unauthorized",
            ))
        }
    }

    async fn add_user_to_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        let list = self
            .get_user_list_by_id(list_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::ListNotFound, "User list not found"))?;

        if list.owner_id != owner_id {
            return Err(DomainError::new(
                ErrorCode::ListUnauthorized,
                "Only owner can modify list",
            ));
        }

        let _ = db_execute!(
            self,
            "INSERT INTO user_list_members (list_id, user_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            list_id,
            user_id,
            Utc::now()
        );

        Ok(true)
    }

    async fn remove_user_from_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        let list = self
            .get_user_list_by_id(list_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::ListNotFound, "User list not found"))?;

        if list.owner_id != owner_id {
            return Err(DomainError::new(
                ErrorCode::ListUnauthorized,
                "Only owner can modify list",
            ));
        }

        let _ = db_execute!(
            self,
            "DELETE FROM user_list_members WHERE list_id = $1 AND user_id = $2",
            list_id,
            user_id
        );

        Ok(true)
    }

    async fn get_user_lists(&self, user_id: Uuid) -> Result<Vec<UserList>, DomainError> {
        let entities: Vec<UserListEntity> = db_fetch_all!(
            self,
            UserListEntity,
            "SELECT * FROM user_lists WHERE owner_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, user_id = %user_id, "Failed to get user lists");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get user lists")
        })?;

        Ok(entities.into_iter().map(UserList::from).collect())
    }

    async fn get_user_list_by_id(&self, list_id: Uuid) -> Result<Option<UserList>, DomainError> {
        let entity: Option<UserListEntity> = db_fetch_optional!(
            self,
            UserListEntity,
            "SELECT * FROM user_lists WHERE id = $1",
            list_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, list_id = %list_id, "Failed to get user list by id");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get user list by id")
        })?;

        Ok(entity.map(UserList::from))
    }

    async fn get_list_members(&self, list_id: Uuid) -> Result<Vec<User>, DomainError> {
        let entities: Vec<UserEntity> = db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u JOIN user_list_members ulm ON u.id = ulm.user_id WHERE ulm.list_id = $1",
            list_id
        )
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, list_id = %list_id, "Failed to get list members");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get list members")
        })?;

        Ok(entities.into_iter().map(User::from).collect())
    }

    async fn get_list_feed_cursor(
        &self,
        list_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError> {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN user_list_members ulm ON p.author_id = ulm.user_id
                 WHERE ulm.list_id = $1
                   AND ((p.created_at < $2) OR (p.created_at = $2 AND p.id < $3))
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $4",
                list_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN user_list_members ulm ON p.author_id = ulm.user_id
                 WHERE ulm.list_id = $1
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $2",
                list_id,
                fetch_limit
            ),
        }
        .map_err(|e| {
            error!(target: "serve::post_repo", error = %e, list_id = %list_id, "Failed to get list feed cursor");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get list feed cursor")
        })?;

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        Ok((items, has_next_page))
    }

    // 3. Post Views & Analytics
    async fn record_post_view(&self, post_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError> {
        let _ = db_execute!(
            self,
            "INSERT INTO post_views (post_id, viewer_id, viewed_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            post_id,
            viewer_id,
            Utc::now()
        );

        let _ = db_execute!(
            self,
            "UPDATE posts SET views_count = views_count + 1 WHERE id = $1",
            post_id
        );

        Ok(true)
    }

    async fn get_post_analytics(&self, post_id: Uuid) -> Result<PostAnalytics, DomainError> {
        let _ = self
            .get_post_by_id(post_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let likes = self.get_likes_count(post_id).await;
        let reposts = self.get_reposts_count(post_id).await;
        let replies: i64 = db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM comments WHERE post_id = $1",
            post_id
        )
        .unwrap_or(0);
        let bookmarks: i64 = db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM bookmarks WHERE post_id = $1",
            post_id
        )
        .unwrap_or(0);
        let views: i64 = db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM post_views WHERE post_id = $1",
            post_id
        )
        .unwrap_or(0);

        let engagement_rate = if views > 0 {
            ((likes + reposts + replies as usize + bookmarks as usize) as f64 / views as f64)
                * 100.0
        } else {
            0.0
        };

        Ok(PostAnalytics {
            post_id,
            views_count: views,
            likes_count: likes,
            reposts_count: reposts,
            comments_count: replies as usize,
            engagement_rate,
        })
    }
}
