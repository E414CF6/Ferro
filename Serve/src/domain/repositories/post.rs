use crate::domain::errors::DomainError;
use crate::domain::models::{
    Comment, Post, PostAnalytics, PostAudience, PostMedia, User, UserList,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait PostRepository: Send + Sync {
    async fn get_user_posts_count(&self, user_id: Uuid) -> usize;
    async fn get_liked_posts_by_user(&self, user_id: Uuid) -> Result<Vec<Post>, DomainError>;
    async fn get_comments_by_user(&self, user_id: Uuid) -> Result<Vec<Comment>, DomainError>;
    async fn get_posts(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError>;
    async fn get_posts_cursor(
        &self,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError>;
    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>, DomainError>;
    async fn get_posts_by_author(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError>;
    async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError>;
    async fn create_post_with_details(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
        media: Vec<PostMedia>,
        poll: Option<(String, Vec<String>, i64)>,
    ) -> Result<Post, DomainError>;
    async fn create_quote_post(
        &self,
        author_id: Uuid,
        quote_post_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError>;
    async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError>;
    async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<bool, DomainError>;
    async fn get_feed(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Post>, DomainError>;
    async fn get_feed_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError>;
    async fn search_posts(&self, query: &str) -> Result<Vec<Post>, DomainError>;
    async fn search_posts_cursor(
        &self,
        query: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError>;
    async fn get_posts_by_hashtag_cursor(
        &self,
        hashtag: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError>;
    async fn get_trending_hashtags(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<(String, usize)>, DomainError>;

    // Likes & Reposts
    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unlike_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn get_likes_count(&self, post_id: Uuid) -> usize;
    async fn is_post_liked_by(&self, post_id: Uuid, user_id: Uuid) -> bool;
    async fn repost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unrepost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn get_reposts_count(&self, post_id: Uuid) -> usize;
    async fn is_post_reposted_by(&self, post_id: Uuid, user_id: Uuid) -> bool;

    // Media Attachments
    async fn add_post_media(
        &self,
        post_id: Uuid,
        media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError>;
    async fn get_post_media(&self, post_id: Uuid) -> Result<Vec<PostMedia>, DomainError>;

    // User Lists & Custom Feeds
    async fn create_user_list(
        &self,
        owner_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
        member_ids: Vec<Uuid>,
    ) -> Result<UserList, DomainError>;
    async fn update_user_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<UserList, DomainError>;
    async fn delete_user_list(&self, list_id: Uuid, owner_id: Uuid) -> Result<bool, DomainError>;
    async fn add_user_to_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn remove_user_from_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn get_user_lists(&self, user_id: Uuid) -> Result<Vec<UserList>, DomainError>;
    async fn get_user_list_by_id(&self, list_id: Uuid) -> Result<Option<UserList>, DomainError>;
    async fn get_list_members(&self, list_id: Uuid) -> Result<Vec<User>, DomainError>;
    async fn get_list_feed_cursor(
        &self,
        list_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<Post>, bool), DomainError>;

    // Post Views & Analytics
    async fn record_post_view(&self, post_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError>;
    async fn get_post_analytics(&self, post_id: Uuid) -> Result<PostAnalytics, DomainError>;
}
