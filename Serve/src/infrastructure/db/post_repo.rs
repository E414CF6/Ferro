use super::postgres::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    BookmarkCollection, Comment, Poll, PollOption, PollVote, Post, PostAnalytics, PostAudience,
    PostMedia, Report, ReportReason, ReportStatus, ReportTargetType, User, UserList,
};
use crate::domain::repositories::PostRepository;
use crate::infrastructure::db::entities::{
    BookmarkCollectionEntity, CommentEntity, HashtagEntity, PollEntity, PollOptionEntity,
    PostEntity, PostMediaEntity, ReportEntity, UserEntity, UserListEntity,
};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
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

    async fn get_liked_posts_by_user(&self, user_id: Uuid) -> Vec<Post> {
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT p.* FROM posts p JOIN likes l ON p.id = l.post_id WHERE l.user_id = $1 ORDER BY l.created_at DESC",
            user_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn get_comments_by_user(&self, user_id: Uuid) -> Vec<Comment> {
        let entities: Vec<CommentEntity> = db_fetch_all!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE author_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Comment::from).collect()
    }

    async fn get_posts(&self, limit: Option<usize>, offset: Option<usize>) -> Vec<Post> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
            lim,
            off
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn get_posts_cursor(
        &self,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
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
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    async fn get_post_by_id(&self, id: Uuid) -> Option<Post> {
        let entity: Option<PostEntity> =
            db_fetch_optional!(self, PostEntity, "SELECT * FROM posts WHERE id = $1", id)
                .ok()
                .flatten();

        entity.map(Post::from)
    }

    async fn get_posts_by_author(&self, author_id: Uuid) -> Vec<Post> {
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts WHERE author_id = $1 ORDER BY created_at DESC, id DESC",
            author_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError> {
        let id = Uuid::new_v4();
        let aud = audience.unwrap_or(PostAudience::Public);
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO posts (id, author_id, content, quote_post_id, pinned_comment_id, audience, views_count, created_at)
             VALUES ($1, $2, $3, NULL, NULL, $4, 0, $5)",
            id,
            author_id,
            &content,
            aud.as_str(),
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create post");
            DomainError::new(ErrorCode::PostCreateFailed, "Failed to create post")
        })?;

        self.index_post_content(id, &content).await;

        Ok(Post {
            id,
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
            .await
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
            .await
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
    ) -> Vec<Post> {
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
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn get_feed_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
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
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    async fn search_posts(&self, query: &str) -> Vec<Post> {
        let search_pattern = format!("%{}%", query);
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT * FROM posts WHERE content LIKE $1 ORDER BY created_at DESC, id DESC LIMIT 50",
            &search_pattern
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn search_posts_cursor(
        &self,
        query: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
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
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    async fn get_posts_by_hashtag_cursor(
        &self,
        hashtag: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
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
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    // Comments
    async fn create_comment(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
    ) -> Result<Comment, DomainError> {
        let _ = self
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        if let Some(pid) = parent_id {
            let parent = self.get_comment_by_id(pid).await.ok_or_else(|| {
                DomainError::new(ErrorCode::ParentCommentNotFound, "Parent comment not found")
            })?;
            if parent.post_id != post_id {
                return Err(DomainError::new(
                    ErrorCode::ParentCommentNotFound,
                    "Parent comment belongs to a different post",
                ));
            }
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO comments (id, post_id, author_id, parent_id, content, is_edited, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, false, $6, $7)",
            id,
            post_id,
            author_id,
            parent_id,
            &content,
            now,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create comment");
            DomainError::new(ErrorCode::CommentCreateFailed, "Failed to create comment")
        })?;

        Ok(Comment {
            id,
            post_id,
            author_id,
            parent_id,
            content,
            is_edited: false,
            created_at: now,
            updated_at: now,
        })
    }

    async fn edit_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
        new_content: String,
    ) -> Result<Comment, DomainError> {
        let trimmed = new_content.trim();
        if trimmed.is_empty() || trimmed.len() > 500 {
            return Err(DomainError::new(
                ErrorCode::CommentContentInvalid,
                "Comment content must be between 1 and 500 characters",
            ));
        }

        let now = Utc::now();
        let res = db_execute!(
            self,
            "UPDATE comments SET content = $1, is_edited = true, updated_at = $2 WHERE id = $3 AND author_id = $4",
            trimmed,
            now,
            comment_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to edit comment");
            DomainError::new(ErrorCode::CommentNotFound, "Comment not found or unauthorized")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::CommentNotFound,
                "Comment not found or unauthorized",
            ));
        }

        self.get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))
    }

    async fn delete_comment(&self, comment_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        let result = db_execute!(
            self,
            "DELETE FROM comments WHERE id = $1 AND author_id = $2",
            comment_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete comment");
            DomainError::new(ErrorCode::CommentDeleteFailed, "Failed to delete comment")
        })?;

        if result.rows_affected() > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::CommentNotFound,
                "Comment not found or unauthorized",
            ))
        }
    }

    async fn get_comment_by_id(&self, comment_id: Uuid) -> Option<Comment> {
        let entity: Option<CommentEntity> = db_fetch_optional!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE id = $1",
            comment_id
        )
        .ok()
        .flatten();

        entity.map(Comment::from)
    }

    async fn get_comments_for_post(&self, post_id: Uuid) -> Vec<Comment> {
        let entities: Vec<CommentEntity> = db_fetch_all!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE post_id = $1 ORDER BY created_at ASC, id ASC",
            post_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Comment::from).collect()
    }

    async fn get_top_level_comments_for_post(&self, post_id: Uuid) -> Vec<Comment> {
        let entities: Vec<CommentEntity> = db_fetch_all!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE post_id = $1 AND parent_id IS NULL ORDER BY created_at ASC, id ASC",
            post_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Comment::from).collect()
    }

    async fn get_comments_cursor(
        &self,
        post_id: Uuid,
        top_level_only: bool,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<CommentEntity> = match (top_level_only, after) {
            (true, Some((after_time, after_id))) => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE post_id = $1 AND parent_id IS NULL
                   AND ((created_at > $2) OR (created_at = $2 AND id > $3))
                 ORDER BY created_at ASC, id ASC
                 LIMIT $4",
                post_id,
                after_time,
                after_id,
                fetch_limit
            ),
            (true, None) => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE post_id = $1 AND parent_id IS NULL
                 ORDER BY created_at ASC, id ASC
                 LIMIT $2",
                post_id,
                fetch_limit
            ),
            (false, Some((after_time, after_id))) => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE post_id = $1
                   AND ((created_at > $2) OR (created_at = $2 AND id > $3))
                 ORDER BY created_at ASC, id ASC
                 LIMIT $4",
                post_id,
                after_time,
                after_id,
                fetch_limit
            ),
            (false, None) => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE post_id = $1
                 ORDER BY created_at ASC, id ASC
                 LIMIT $2",
                post_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Comment> = entities
            .into_iter()
            .take(first)
            .map(Comment::from)
            .collect();
        (items, has_next_page)
    }

    async fn get_replies_for_comment(&self, comment_id: Uuid) -> Vec<Comment> {
        let entities: Vec<CommentEntity> = db_fetch_all!(
            self,
            CommentEntity,
            "SELECT * FROM comments WHERE parent_id = $1 ORDER BY created_at ASC, id ASC",
            comment_id
        )
        .unwrap_or_default();

        entities.into_iter().map(Comment::from).collect()
    }

    async fn get_replies_cursor(
        &self,
        comment_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<CommentEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE parent_id = $1
                   AND ((created_at > $2) OR (created_at = $2 AND id > $3))
                 ORDER BY created_at ASC, id ASC
                 LIMIT $4",
                comment_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                CommentEntity,
                "SELECT * FROM comments
                 WHERE parent_id = $1
                 ORDER BY created_at ASC, id ASC
                 LIMIT $2",
                comment_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Comment> = entities
            .into_iter()
            .take(first)
            .map(Comment::from)
            .collect();
        (items, has_next_page)
    }

    async fn get_replies_count(&self, comment_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM comments WHERE parent_id = $1",
            comment_id
        )
        .unwrap_or(0) as usize
    }

    async fn like_comment(&self, user_id: Uuid, comment_id: Uuid) -> Result<Comment, DomainError> {
        let comment = self
            .get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))?;

        let _ = db_execute!(
            self,
            "INSERT INTO comment_likes (user_id, comment_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            comment_id,
            Utc::now()
        );

        Ok(comment)
    }

    async fn unlike_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Comment, DomainError> {
        let comment = self
            .get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))?;

        let _ = db_execute!(
            self,
            "DELETE FROM comment_likes WHERE user_id = $1 AND comment_id = $2",
            user_id,
            comment_id
        );

        Ok(comment)
    }

    async fn get_comment_likes_count(&self, comment_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM comment_likes WHERE comment_id = $1",
            comment_id
        )
        .unwrap_or(0) as usize
    }

    async fn is_comment_liked_by(&self, comment_id: Uuid, user_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM comment_likes WHERE user_id = $1 AND comment_id = $2)",
            user_id,
            comment_id
        )
        .unwrap_or(false)
    }

    async fn pin_comment(
        &self,
        post_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    ) -> Result<Post, DomainError> {
        let comment = self
            .get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))?;

        if comment.post_id != post_id {
            return Err(DomainError::new(
                ErrorCode::CommentNotFound,
                "Comment belongs to a different post",
            ));
        }

        let res = db_execute!(
            self,
            "UPDATE posts SET pinned_comment_id = $1 WHERE id = $2 AND author_id = $3",
            comment_id,
            post_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to pin comment");
            DomainError::new(ErrorCode::PostNotFound, "Failed to pin comment")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::PostNotFound,
                "Post not found or unauthorized",
            ));
        }

        self.get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))
    }

    async fn unpin_comment(&self, post_id: Uuid, author_id: Uuid) -> Result<Post, DomainError> {
        let res = db_execute!(
            self,
            "UPDATE posts SET pinned_comment_id = NULL WHERE id = $1 AND author_id = $2",
            post_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to unpin comment");
            DomainError::new(ErrorCode::PostNotFound, "Failed to unpin comment")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::PostNotFound,
                "Post not found or unauthorized",
            ));
        }

        self.get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))
    }

    async fn get_pinned_comment(&self, post_id: Uuid) -> Option<Comment> {
        let post = self.get_post_by_id(post_id).await?;
        let pinned_id = post.pinned_comment_id?;
        self.get_comment_by_id(pinned_id).await
    }

    // Likes & Reposts & Bookmarks
    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await
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
            .await
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
            .await
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
            .await
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

    async fn save_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "INSERT INTO bookmarks (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            post_id,
            Utc::now()
        );

        Ok(post)
    }

    async fn unsave_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "DELETE FROM bookmarks WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        );

        Ok(post)
    }

    async fn is_post_saved_by(&self, post_id: Uuid, user_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND post_id = $2)",
            user_id,
            post_id
        )
        .unwrap_or(false)
    }

    async fn get_saved_posts(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Post> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT p.* FROM posts p JOIN bookmarks b ON p.id = b.post_id WHERE b.user_id = $1 ORDER BY b.created_at DESC LIMIT $2 OFFSET $3",
            user_id,
            lim,
            off
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn get_saved_posts_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN bookmarks b ON p.id = b.post_id
                 WHERE b.user_id = $1
                   AND ((b.created_at < $2) OR (b.created_at = $2 AND p.id < $3))
                 ORDER BY b.created_at DESC, p.id DESC
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
                 JOIN bookmarks b ON p.id = b.post_id
                 WHERE b.user_id = $1
                 ORDER BY b.created_at DESC, p.id DESC
                 LIMIT $2",
                user_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    async fn get_trending_hashtags(&self, limit: Option<usize>) -> Vec<(String, usize)> {
        let lim = limit.unwrap_or(10) as i64;
        let entities: Vec<HashtagEntity> = db_fetch_all!(
            self,
            HashtagEntity,
            "SELECT * FROM hashtags ORDER BY posts_count DESC LIMIT $1",
            lim
        )
        .unwrap_or_default();

        entities
            .into_iter()
            .map(|h| (h.name, h.posts_count as usize))
            .collect()
    }

    // 1. Media Attachments
    async fn add_post_media(
        &self,
        post_id: Uuid,
        media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError> {
        let _ = self
            .get_post_by_id(post_id)
            .await
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

    async fn get_post_media(&self, post_id: Uuid) -> Vec<PostMedia> {
        let entities: Vec<PostMediaEntity> = db_fetch_all!(
            self,
            PostMediaEntity,
            "SELECT * FROM post_media WHERE post_id = $1 ORDER BY sort_order ASC, created_at ASC",
            post_id
        )
        .unwrap_or_default();

        entities.into_iter().map(PostMedia::from).collect()
    }

    // 2. Polls & Voting
    async fn create_poll(
        &self,
        post_id: Uuid,
        question: String,
        options: Vec<String>,
        duration_seconds: i64,
    ) -> Result<Poll, DomainError> {
        let poll_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(duration_seconds);

        db_execute!(
            self,
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
            db_execute!(
                self,
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

        Ok(Poll {
            id: poll_id,
            post_id,
            question,
            expires_at,
            created_at: now,
        })
    }

    async fn get_poll_by_post_id(&self, post_id: Uuid) -> Option<Poll> {
        let entity: Option<PollEntity> = db_fetch_optional!(
            self,
            PollEntity,
            "SELECT * FROM polls WHERE post_id = $1",
            post_id
        )
        .ok()
        .flatten();

        entity.map(Poll::from)
    }

    async fn get_poll_by_id(&self, poll_id: Uuid) -> Option<Poll> {
        let entity: Option<PollEntity> = db_fetch_optional!(
            self,
            PollEntity,
            "SELECT * FROM polls WHERE id = $1",
            poll_id
        )
        .ok()
        .flatten();

        entity.map(Poll::from)
    }

    async fn get_poll_options(&self, poll_id: Uuid) -> Vec<PollOption> {
        let entities: Vec<PollOptionEntity> = db_fetch_all!(
            self,
            PollOptionEntity,
            "SELECT * FROM poll_options WHERE poll_id = $1 ORDER BY sort_order ASC",
            poll_id
        )
        .unwrap_or_default();

        entities.into_iter().map(PollOption::from).collect()
    }

    async fn vote_poll(
        &self,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollVote, DomainError> {
        let poll = self
            .get_poll_by_id(poll_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PollNotFound, "Poll not found"))?;

        if Utc::now() > poll.expires_at {
            return Err(DomainError::new(ErrorCode::PollExpired, "Poll has expired"));
        }

        let now = Utc::now();
        let res = db_execute!(
            self,
            "INSERT INTO poll_votes (poll_id, option_id, user_id, created_at) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            poll_id,
            option_id,
            user_id,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to cast vote on poll");
            DomainError::new(ErrorCode::PollAlreadyVoted, "Failed to cast vote")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::PollAlreadyVoted,
                "User has already voted in this poll",
            ));
        }

        Ok(PollVote {
            poll_id,
            option_id,
            user_id,
            created_at: now,
        })
    }

    async fn get_poll_option_votes_count(&self, option_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM poll_votes WHERE option_id = $1",
            option_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_poll_total_votes(&self, poll_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM poll_votes WHERE poll_id = $1",
            poll_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_user_vote_for_poll(&self, poll_id: Uuid, user_id: Uuid) -> Option<Uuid> {
        db_scalar!(
            self,
            Uuid,
            "SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2",
            poll_id,
            user_id
        )
        .ok()
    }

    // 3. User Lists & Custom Feeds
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
            .await
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
            .await
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
            .await
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
            .await
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

    async fn get_user_lists(&self, user_id: Uuid) -> Vec<UserList> {
        let entities: Vec<UserListEntity> = db_fetch_all!(
            self,
            UserListEntity,
            "SELECT * FROM user_lists WHERE owner_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .unwrap_or_default();

        entities.into_iter().map(UserList::from).collect()
    }

    async fn get_user_list_by_id(&self, list_id: Uuid) -> Option<UserList> {
        let entity: Option<UserListEntity> = db_fetch_optional!(
            self,
            UserListEntity,
            "SELECT * FROM user_lists WHERE id = $1",
            list_id
        )
        .ok()
        .flatten();

        entity.map(UserList::from)
    }

    async fn get_list_members(&self, list_id: Uuid) -> Vec<User> {
        let entities: Vec<UserEntity> = db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u JOIN user_list_members ulm ON u.id = ulm.user_id WHERE ulm.list_id = $1",
            list_id
        )
        .unwrap_or_default();

        entities.into_iter().map(User::from).collect()
    }

    async fn get_list_feed_cursor(
        &self,
        list_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
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
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    // 4. Bookmark Collections
    async fn create_bookmark_collection(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
    ) -> Result<BookmarkCollection, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO bookmark_collections (id, user_id, name, description, is_private, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            id,
            user_id,
            &name,
            &description,
            is_private,
            now,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create bookmark collection");
            DomainError::new(ErrorCode::CollectionUnauthorized, "Failed to create collection")
        })?;

        Ok(BookmarkCollection {
            id,
            user_id,
            name,
            description,
            is_private,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<BookmarkCollection, DomainError> {
        let current = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if current.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can edit collection",
            ));
        }

        let new_name = name.unwrap_or(current.name);
        let new_desc = description.or(current.description);
        let new_private = is_private.unwrap_or(current.is_private);
        let now = Utc::now();

        let res = db_execute!(
            self,
            "UPDATE bookmark_collections SET name = $1, description = $2, is_private = $3, updated_at = $4 WHERE id = $5 AND user_id = $6",
            &new_name,
            &new_desc,
            new_private,
            now,
            collection_id,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update collection");
            DomainError::new(ErrorCode::CollectionUnauthorized, "Failed to update collection")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::CollectionNotFound,
                "Collection not found",
            ));
        }

        self.get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CollectionNotFound, "Collection not found"))
    }

    async fn delete_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = db_execute!(
            self,
            "DELETE FROM bookmark_collections WHERE id = $1 AND user_id = $2",
            collection_id,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete collection");
            DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Failed to delete collection",
            )
        })?;

        if result.rows_affected() > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::CollectionNotFound,
                "Collection not found or unauthorized",
            ))
        }
    }

    async fn add_post_to_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        let col = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if col.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can modify collection",
            ));
        }

        let _ = db_execute!(
            self,
            "INSERT INTO collection_bookmarks (collection_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            collection_id,
            post_id,
            Utc::now()
        );

        Ok(true)
    }

    async fn remove_post_from_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        let col = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if col.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can modify collection",
            ));
        }

        let _ = db_execute!(
            self,
            "DELETE FROM collection_bookmarks WHERE collection_id = $1 AND post_id = $2",
            collection_id,
            post_id
        );

        Ok(true)
    }

    async fn get_user_collections(&self, user_id: Uuid) -> Vec<BookmarkCollection> {
        let entities: Vec<BookmarkCollectionEntity> = db_fetch_all!(
            self,
            BookmarkCollectionEntity,
            "SELECT * FROM bookmark_collections WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .unwrap_or_default();

        entities.into_iter().map(BookmarkCollection::from).collect()
    }

    async fn get_collection_by_id(&self, collection_id: Uuid) -> Option<BookmarkCollection> {
        let entity: Option<BookmarkCollectionEntity> = db_fetch_optional!(
            self,
            BookmarkCollectionEntity,
            "SELECT * FROM bookmark_collections WHERE id = $1",
            collection_id
        )
        .ok()
        .flatten();

        entity.map(BookmarkCollection::from)
    }

    async fn get_collection_posts_cursor(
        &self,
        collection_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN collection_bookmarks cb ON p.id = cb.post_id
                 WHERE cb.collection_id = $1
                   AND ((p.created_at < $2) OR (p.created_at = $2 AND p.id < $3))
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $4",
                collection_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN collection_bookmarks cb ON p.id = cb.post_id
                 WHERE cb.collection_id = $1
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $2",
                collection_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    // 5. Reports & Moderation
    async fn create_report(
        &self,
        reporter_id: Uuid,
        target_type: ReportTargetType,
        target_id: Uuid,
        reason: ReportReason,
        details: Option<String>,
    ) -> Result<Report, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO reports (id, reporter_id, target_type, target_id, reason, details, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            id,
            reporter_id,
            target_type.as_str(),
            target_id,
            reason.as_str(),
            &details,
            ReportStatus::Pending.as_str(),
            now,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create report");
            DomainError::new(ErrorCode::ReportNotFound, "Failed to create report")
        })?;

        Ok(Report {
            id,
            reporter_id,
            target_type,
            target_id,
            reason,
            details,
            status: ReportStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    async fn resolve_report(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Report, DomainError> {
        let now = Utc::now();
        let res = db_execute!(
            self,
            "UPDATE reports SET status = $1, updated_at = $2 WHERE id = $3",
            status.as_str(),
            now,
            report_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to resolve report");
            DomainError::new(ErrorCode::ReportNotFound, "Failed to resolve report")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::ReportNotFound,
                "Report not found",
            ));
        }

        let entity: Option<ReportEntity> = db_fetch_optional!(
            self,
            ReportEntity,
            "SELECT * FROM reports WHERE id = $1",
            report_id
        )
        .ok()
        .flatten();

        entity
            .map(Report::from)
            .ok_or_else(|| DomainError::new(ErrorCode::ReportNotFound, "Report not found"))
    }

    async fn get_reports(&self, status: Option<ReportStatus>, limit: Option<usize>) -> Vec<Report> {
        let lim = limit.unwrap_or(50) as i64;
        let entities: Vec<ReportEntity> = match status {
            Some(st) => db_fetch_all!(
                self,
                ReportEntity,
                "SELECT * FROM reports WHERE status = $1 ORDER BY created_at DESC LIMIT $2",
                st.as_str(),
                lim
            ),
            None => db_fetch_all!(
                self,
                ReportEntity,
                "SELECT * FROM reports ORDER BY created_at DESC LIMIT $1",
                lim
            ),
        }
        .unwrap_or_default();

        entities.into_iter().map(Report::from).collect()
    }

    // 6. Post Views & Analytics
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
            .await
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
