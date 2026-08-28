use crate::domain::errors::DomainError;
use crate::domain::models::{Story, User};

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait StoryRepository: Send + Sync {
    async fn create_story(
        &self,
        author_id: Uuid,
        media_url: String,
        caption: Option<String>,
    ) -> Result<Story, DomainError>;
    async fn get_story_by_id(&self, id: Uuid) -> Option<Story>;
    async fn get_active_stories_for_user(&self, author_id: Uuid) -> Vec<Story>;
    async fn get_stories_feed_for_user(&self, user_id: Uuid) -> Vec<Story>;
    async fn delete_story(&self, story_id: Uuid, author_id: Uuid) -> Result<bool, DomainError>;
    async fn view_story(&self, story_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError>;
    async fn get_story_views_count(&self, story_id: Uuid) -> usize;
    async fn is_story_viewed_by(&self, story_id: Uuid, viewer_id: Uuid) -> bool;
    async fn get_story_viewers(&self, story_id: Uuid) -> Vec<User>;
    async fn has_active_stories(&self, author_id: Uuid) -> bool;
}
