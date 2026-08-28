#![allow(dead_code)]
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Post, PostAnalytics, PostAudience, PostMedia};
use crate::domain::repositories::PostRepository;
use crate::domain::validation::validate_post_content;
use crate::infrastructure::db::postgres::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating Post operations, validation, feeds, and analytics
#[derive(Clone)]
pub struct PostService {
    db: Arc<Database>,
}

impl PostService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError> {
        validate_post_content(&content)?;
        self.db.create_post(author_id, content, audience).await
    }

    pub async fn create_quote_post(
        &self,
        author_id: Uuid,
        quote_post_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        validate_post_content(&content)?;
        self.db
            .create_quote_post(author_id, quote_post_id, content)
            .await
    }

    pub async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        validate_post_content(&content)?;
        self.db.update_post(post_id, author_id, content).await
    }

    pub async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        self.db.delete_post(post_id, author_id).await
    }

    pub async fn get_post_by_id(&self, post_id: Uuid) -> Option<Post> {
        self.db.get_post_by_id(post_id).await
    }

    pub async fn get_posts_cursor(
        &self,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.db.get_posts_cursor(first, after).await
    }

    pub async fn get_feed_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.db.get_feed_cursor(user_id, first, after).await
    }

    pub async fn get_posts_by_hashtag_cursor(
        &self,
        hashtag: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.db
            .get_posts_by_hashtag_cursor(hashtag, first, after)
            .await
    }

    pub async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.db.like_post(user_id, post_id).await
    }

    pub async fn unlike_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.db.unlike_post(user_id, post_id).await
    }

    pub async fn repost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.db.repost_post(user_id, post_id).await
    }

    pub async fn unrepost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.db.unrepost_post(user_id, post_id).await
    }

    pub async fn add_post_media(
        &self,
        post_id: Uuid,
        media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError> {
        self.db.add_post_media(post_id, media).await
    }

    pub async fn get_post_media(&self, post_id: Uuid) -> Vec<PostMedia> {
        self.db.get_post_media(post_id).await
    }

    pub async fn record_post_view(
        &self,
        post_id: Uuid,
        viewer_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.record_post_view(post_id, viewer_id).await
    }

    pub async fn get_post_analytics(
        &self,
        post_id: Uuid,
        requester_id: Uuid,
    ) -> Result<PostAnalytics, DomainError> {
        let post = self
            .db
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        if post.author_id != requester_id {
            return Err(DomainError::new(
                ErrorCode::ErrorForbidden,
                "Only the author can view post analytics",
            ));
        }

        self.db.get_post_analytics(post_id).await
    }

    pub async fn get_trending_hashtags(&self, limit: Option<usize>) -> Vec<(String, usize)> {
        self.db.get_trending_hashtags(limit).await
    }
}
