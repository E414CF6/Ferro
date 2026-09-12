#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{Comment, Post};
use crate::domain::repositories::CommentRepository;
use crate::domain::validation::validate_comment_content;
use crate::infrastructure::db::database::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating Comment operations, nested replies, liking, and pinning
#[derive(Clone)]
pub struct CommentService<R: CommentRepository = Database> {
    repo: Arc<R>,
}

impl<R: CommentRepository> CommentService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn create_comment(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
    ) -> Result<Comment, DomainError> {
        validate_comment_content(&content)?;
        self.repo
            .create_comment(post_id, author_id, content, parent_id)
            .await
    }

    pub async fn edit_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
        new_content: String,
    ) -> Result<Comment, DomainError> {
        validate_comment_content(&new_content)?;
        self.repo
            .edit_comment(comment_id, author_id, new_content)
            .await
    }

    pub async fn delete_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo.delete_comment(comment_id, author_id).await
    }

    pub async fn get_comment_by_id(&self, comment_id: Uuid) -> Option<Comment> {
        self.repo.get_comment_by_id(comment_id).await
    }

    pub async fn get_comments_cursor(
        &self,
        post_id: Uuid,
        top_level_only: bool,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        self.repo
            .get_comments_cursor(post_id, top_level_only, first, after)
            .await
    }

    pub async fn get_replies_cursor(
        &self,
        comment_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        self.repo.get_replies_cursor(comment_id, first, after).await
    }

    pub async fn like_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Comment, DomainError> {
        self.repo.like_comment(user_id, comment_id).await
    }

    pub async fn unlike_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Comment, DomainError> {
        self.repo.unlike_comment(user_id, comment_id).await
    }

    pub async fn pin_comment(
        &self,
        post_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    ) -> Result<Post, DomainError> {
        self.repo.pin_comment(post_id, comment_id, author_id).await
    }

    pub async fn unpin_comment(&self, post_id: Uuid, author_id: Uuid) -> Result<Post, DomainError> {
        self.repo.unpin_comment(post_id, author_id).await
    }
}
