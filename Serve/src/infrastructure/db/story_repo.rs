use super::postgres::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Story, User};
use crate::domain::repositories::StoryRepository;
use crate::infrastructure::db::entities::{StoryEntity, UserEntity};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl StoryRepository for Database {
    async fn create_story(
        &self,
        author_id: Uuid,
        media_url: String,
        caption: Option<String>,
    ) -> Result<Story, DomainError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);
        let story = Story {
            id: Uuid::new_v4(),
            author_id,
            media_url,
            caption,
            created_at: now,
            expires_at,
        };

        db_execute!(
            self,
            "INSERT INTO stories (id, author_id, media_url, caption, created_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6)",
            story.id,
            story.author_id,
            &story.media_url,
            &story.caption,
            story.created_at,
            story.expires_at
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create story");
            DomainError::new(ErrorCode::StoryCreateFailed, ErrorCode::StoryCreateFailed.as_str())
        })?;

        Ok(story)
    }

    async fn get_story_by_id(&self, id: Uuid) -> Result<Option<Story>, DomainError> {
        let entity = db_fetch_optional!(
            self,
            StoryEntity,
            "SELECT * FROM stories WHERE id = $1",
            id
        )
        .map_err(|e| {
            error!(target: "serve::story_repo", error = %e, id = %id, "Failed to get story by id");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get story by id")
        })?;

        Ok(entity.map(Story::from))
    }

    async fn get_active_stories_for_user(
        &self,
        author_id: Uuid,
    ) -> Result<Vec<Story>, DomainError> {
        let now = Utc::now();
        let entities = db_fetch_all!(
            self,
            StoryEntity,
            "SELECT * FROM stories WHERE author_id = $1 AND expires_at > $2 ORDER BY created_at",
            author_id,
            now
        )
        .map_err(|e| {
            error!(target: "serve::story_repo", error = %e, author_id = %author_id, "Failed to get active stories");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get active stories")
        })?;

        Ok(entities.into_iter().map(Story::from).collect())
    }

    async fn get_stories_feed_for_user(&self, user_id: Uuid) -> Result<Vec<Story>, DomainError> {
        let now = Utc::now();
        let entities = db_fetch_all!(
            self,
            StoryEntity,
            "SELECT s.* FROM stories s WHERE s.expires_at > $1 AND (s.author_id = $2 OR s.author_id IN (SELECT followee_id FROM follows WHERE follower_id = $2)) ORDER BY s.created_at DESC",
            now,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::story_repo", error = %e, user_id = %user_id, "Failed to get stories feed");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get stories feed")
        })?;

        Ok(entities.into_iter().map(Story::from).collect())
    }

    async fn delete_story(&self, story_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "DELETE FROM stories WHERE id = $1 AND author_id = $2",
            story_id,
            author_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete story");
            DomainError::new(
                ErrorCode::StoryDeleteFailed,
                ErrorCode::StoryDeleteFailed.as_str(),
            )
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::StoryNotFound,
                ErrorCode::StoryNotFound.as_str(),
            ))
        }
    }

    async fn view_story(&self, story_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError> {
        let _ = db_execute!(
            self,
            "INSERT INTO story_views (story_id, viewer_id, viewed_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            story_id,
            viewer_id,
            Utc::now()
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to view story");
            DomainError::new(ErrorCode::StoryViewFailed, ErrorCode::StoryViewFailed.as_str())
        })?;

        Ok(true)
    }

    async fn get_story_views_count(&self, story_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM story_views WHERE story_id = $1",
            story_id
        )
        .unwrap_or(0) as usize
    }

    async fn is_story_viewed_by(&self, story_id: Uuid, viewer_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM story_views WHERE story_id = $1 AND viewer_id = $2)",
            story_id,
            viewer_id
        )
        .unwrap_or(false)
    }

    async fn get_story_viewers(&self, story_id: Uuid) -> Result<Vec<User>, DomainError> {
        let entities = db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u JOIN story_views v ON u.id = v.viewer_id WHERE v.story_id = $1 ORDER BY v.viewed_at DESC",
            story_id
        )
        .map_err(|e| {
            error!(target: "serve::story_repo", error = %e, story_id = %story_id, "Failed to get story viewers");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to get story viewers")
        })?;

        Ok(entities.into_iter().map(User::from).collect())
    }

    async fn has_active_stories(&self, author_id: Uuid) -> bool {
        let now = Utc::now();
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM stories WHERE author_id = $1 AND expires_at > $2)",
            author_id,
            now
        )
        .unwrap_or(false)
    }

    async fn prune_expired_stories(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, DomainError> {
        let _ = db_execute!(
            self,
            "DELETE FROM story_views WHERE story_id IN (SELECT id FROM stories WHERE expires_at < $1)",
            older_than
        );
        let rows = db_execute!(
            self,
            "DELETE FROM stories WHERE expires_at < $1",
            older_than
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to prune expired stories");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to prune expired stories")
        })?;
        Ok(rows.rows_affected())
    }
}
