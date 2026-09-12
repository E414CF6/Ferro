#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{Story, User};
use crate::domain::repositories::StoryRepository;
use crate::infrastructure::db::database::Database;
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating ephemeral 24-hour Stories, views, and viewer lists
#[derive(Clone)]
pub struct StoryService<R: StoryRepository = Database> {
    repo: Arc<R>,
}

impl<R: StoryRepository> StoryService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn create_story(
        &self,
        author_id: Uuid,
        media_url: String,
        caption: Option<String>,
    ) -> Result<Story, DomainError> {
        self.repo.create_story(author_id, media_url, caption).await
    }

    pub async fn get_story_by_id(&self, story_id: Uuid) -> Option<Story> {
        self.repo.get_story_by_id(story_id).await
    }

    pub async fn get_active_stories_for_user(&self, author_id: Uuid) -> Vec<Story> {
        self.repo.get_active_stories_for_user(author_id).await
    }

    pub async fn get_stories_feed_for_user(&self, user_id: Uuid) -> Vec<Story> {
        self.repo.get_stories_feed_for_user(user_id).await
    }

    pub async fn delete_story(&self, story_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        self.repo.delete_story(story_id, author_id).await
    }

    pub async fn view_story(&self, story_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError> {
        self.repo.view_story(story_id, viewer_id).await
    }

    pub async fn get_story_viewers(&self, story_id: Uuid) -> Vec<User> {
        self.repo.get_story_viewers(story_id).await
    }

    pub async fn has_active_stories(&self, author_id: Uuid) -> bool {
        self.repo.has_active_stories(author_id).await
    }
}
