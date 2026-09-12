#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{BookmarkCollection, Post};
use crate::domain::repositories::BookmarkRepository;
use crate::infrastructure::db::database::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating Saved Posts (Bookmarks) and custom Collections
#[derive(Clone)]
pub struct BookmarkService<R: BookmarkRepository = Database> {
    repo: Arc<R>,
}

impl<R: BookmarkRepository> BookmarkService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn save_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.repo.save_post(user_id, post_id).await
    }

    pub async fn unsave_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        self.repo.unsave_post(user_id, post_id).await
    }

    pub async fn get_saved_posts_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.repo.get_saved_posts_cursor(user_id, first, after).await
    }

    pub async fn is_post_saved_by(&self, post_id: Uuid, user_id: Uuid) -> bool {
        self.repo.is_post_saved_by(post_id, user_id).await
    }

    pub async fn create_bookmark_collection(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
    ) -> Result<BookmarkCollection, DomainError> {
        self.repo
            .create_bookmark_collection(user_id, name, description, is_private)
            .await
    }

    pub async fn update_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<BookmarkCollection, DomainError> {
        self.repo
            .update_bookmark_collection(collection_id, user_id, name, description, is_private)
            .await
    }

    pub async fn delete_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo
            .delete_bookmark_collection(collection_id, user_id)
            .await
    }

    pub async fn add_post_to_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo
            .add_post_to_collection(collection_id, user_id, post_id)
            .await
    }

    pub async fn remove_post_from_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo
            .remove_post_from_collection(collection_id, user_id, post_id)
            .await
    }

    pub async fn get_user_collections(&self, user_id: Uuid) -> Vec<BookmarkCollection> {
        self.repo.get_user_collections(user_id).await
    }

    pub async fn get_collection_by_id(&self, collection_id: Uuid) -> Option<BookmarkCollection> {
        self.repo.get_collection_by_id(collection_id).await
    }

    pub async fn get_collection_posts_cursor(
        &self,
        collection_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.repo
            .get_collection_posts_cursor(collection_id, first, after)
            .await
    }
}
