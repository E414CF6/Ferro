use crate::domain::errors::DomainError;
use crate::domain::models::{BookmarkCollection, Post};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait BookmarkRepository: Send + Sync {
    // Post bookmarks
    async fn save_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unsave_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn is_post_saved_by(&self, post_id: Uuid, user_id: Uuid) -> bool;
    async fn get_saved_posts(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Post>;
    async fn get_saved_posts_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);

    // Bookmark collections
    async fn create_bookmark_collection(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
    ) -> Result<BookmarkCollection, DomainError>;
    async fn update_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<BookmarkCollection, DomainError>;
    async fn delete_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn add_post_to_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn remove_post_from_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn get_user_collections(&self, user_id: Uuid) -> Vec<BookmarkCollection>;
    async fn get_collection_by_id(&self, collection_id: Uuid) -> Option<BookmarkCollection>;
    async fn get_collection_posts_cursor(
        &self,
        collection_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);
}
