use super::database::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Comment, Post};
use crate::domain::repositories::{CommentRepository, PostRepository};
use crate::infrastructure::db::entities::CommentEntity;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl CommentRepository for Database {
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
}
