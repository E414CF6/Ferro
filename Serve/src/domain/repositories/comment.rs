use crate::domain::errors::DomainError;
use crate::domain::models::{Comment, Post};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait CommentRepository: Send + Sync {
    async fn create_comment(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
    ) -> Result<Comment, DomainError>;
    async fn edit_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
        new_content: String,
    ) -> Result<Comment, DomainError>;
    async fn delete_comment(&self, comment_id: Uuid, author_id: Uuid) -> Result<bool, DomainError>;
    async fn get_comment_by_id(&self, comment_id: Uuid) -> Option<Comment>;
    async fn get_comments_for_post(&self, post_id: Uuid) -> Vec<Comment>;
    async fn get_top_level_comments_for_post(&self, post_id: Uuid) -> Vec<Comment>;
    async fn get_comments_cursor(
        &self,
        post_id: Uuid,
        top_level_only: bool,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool);
    async fn get_replies_for_comment(&self, comment_id: Uuid) -> Vec<Comment>;
    async fn get_replies_cursor(
        &self,
        comment_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool);
    async fn get_replies_count(&self, comment_id: Uuid) -> usize;
    async fn like_comment(&self, user_id: Uuid, comment_id: Uuid) -> Result<Comment, DomainError>;
    async fn unlike_comment(&self, user_id: Uuid, comment_id: Uuid)
    -> Result<Comment, DomainError>;
    async fn get_comment_likes_count(&self, comment_id: Uuid) -> usize;
    async fn is_comment_liked_by(&self, comment_id: Uuid, user_id: Uuid) -> bool;
    async fn pin_comment(
        &self,
        post_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    ) -> Result<Post, DomainError>;
    async fn unpin_comment(&self, post_id: Uuid, author_id: Uuid) -> Result<Post, DomainError>;
    async fn get_pinned_comment(&self, post_id: Uuid) -> Option<Comment>;
}
