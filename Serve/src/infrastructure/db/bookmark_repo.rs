use super::database::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{BookmarkCollection, Post};
use crate::domain::repositories::{BookmarkRepository, PostRepository};
use crate::infrastructure::db::entities::{BookmarkCollectionEntity, PostEntity};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl BookmarkRepository for Database {
    async fn save_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "INSERT INTO bookmarks (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            user_id,
            post_id,
            Utc::now()
        );

        Ok(post)
    }

    async fn unsave_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError> {
        let post = self
            .get_post_by_id(post_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PostNotFound, "Post not found"))?;

        let _ = db_execute!(
            self,
            "DELETE FROM bookmarks WHERE user_id = $1 AND post_id = $2",
            user_id,
            post_id
        );

        Ok(post)
    }

    async fn is_post_saved_by(&self, post_id: Uuid, user_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND post_id = $2)",
            user_id,
            post_id
        )
        .unwrap_or(false)
    }

    async fn get_saved_posts(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Post> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        let entities: Vec<PostEntity> = db_fetch_all!(
            self,
            PostEntity,
            "SELECT p.* FROM posts p JOIN bookmarks b ON p.id = b.post_id WHERE b.user_id = $1 ORDER BY b.created_at DESC LIMIT $2 OFFSET $3",
            user_id,
            lim,
            off
        )
        .unwrap_or_default();

        entities.into_iter().map(Post::from).collect()
    }

    async fn get_saved_posts_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN bookmarks b ON p.id = b.post_id
                 WHERE b.user_id = $1
                   AND ((b.created_at < $2) OR (b.created_at = $2 AND p.id < $3))
                 ORDER BY b.created_at DESC, p.id DESC
                 LIMIT $4",
                user_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN bookmarks b ON p.id = b.post_id
                 WHERE b.user_id = $1
                 ORDER BY b.created_at DESC, p.id DESC
                 LIMIT $2",
                user_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }

    async fn create_bookmark_collection(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
    ) -> Result<BookmarkCollection, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO bookmark_collections (id, user_id, name, description, is_private, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            id,
            user_id,
            &name,
            &description,
            is_private,
            now,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create bookmark collection");
            DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Failed to create bookmark collection",
            )
        })?;

        Ok(BookmarkCollection {
            id,
            user_id,
            name,
            description,
            is_private,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<BookmarkCollection, DomainError> {
        let current = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if current.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can edit collection",
            ));
        }

        let new_name = name.unwrap_or(current.name);
        let new_desc = description.or(current.description);
        let new_private = is_private.unwrap_or(current.is_private);
        let now = Utc::now();

        let res = db_execute!(
            self,
            "UPDATE bookmark_collections SET name = $1, description = $2, is_private = $3, updated_at = $4 WHERE id = $5 AND user_id = $6",
            &new_name,
            &new_desc,
            new_private,
            now,
            collection_id,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update collection");
            DomainError::new(ErrorCode::CollectionUnauthorized, "Failed to update collection")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::CollectionNotFound,
                "Collection not found",
            ));
        }

        self.get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CollectionNotFound, "Collection not found"))
    }

    async fn delete_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = db_execute!(
            self,
            "DELETE FROM bookmark_collections WHERE id = $1 AND user_id = $2",
            collection_id,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete collection");
            DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Failed to delete collection",
            )
        })?;

        if result.rows_affected() > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::CollectionNotFound,
                "Collection not found or unauthorized",
            ))
        }
    }

    async fn add_post_to_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        let col = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if col.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can modify collection",
            ));
        }

        let _ = db_execute!(
            self,
            "INSERT INTO collection_bookmarks (collection_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            collection_id,
            post_id,
            Utc::now()
        );

        Ok(true)
    }

    async fn remove_post_from_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError> {
        let col = self
            .get_collection_by_id(collection_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::CollectionNotFound, "Collection not found")
            })?;

        if col.user_id != user_id {
            return Err(DomainError::new(
                ErrorCode::CollectionUnauthorized,
                "Only owner can modify collection",
            ));
        }

        let _ = db_execute!(
            self,
            "DELETE FROM collection_bookmarks WHERE collection_id = $1 AND post_id = $2",
            collection_id,
            post_id
        );

        Ok(true)
    }

    async fn get_user_collections(&self, user_id: Uuid) -> Vec<BookmarkCollection> {
        let entities: Vec<BookmarkCollectionEntity> = db_fetch_all!(
            self,
            BookmarkCollectionEntity,
            "SELECT * FROM bookmark_collections WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .unwrap_or_default();

        entities.into_iter().map(BookmarkCollection::from).collect()
    }

    async fn get_collection_by_id(&self, collection_id: Uuid) -> Option<BookmarkCollection> {
        let entity: Option<BookmarkCollectionEntity> = db_fetch_optional!(
            self,
            BookmarkCollectionEntity,
            "SELECT * FROM bookmark_collections WHERE id = $1",
            collection_id
        )
        .ok()
        .flatten();

        entity.map(BookmarkCollection::from)
    }

    async fn get_collection_posts_cursor(
        &self,
        collection_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities: Vec<PostEntity> = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN collection_bookmarks cb ON p.id = cb.post_id
                 WHERE cb.collection_id = $1
                   AND ((p.created_at < $2) OR (p.created_at = $2 AND p.id < $3))
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $4",
                collection_id,
                after_time,
                after_id,
                fetch_limit
            ),
            None => db_fetch_all!(
                self,
                PostEntity,
                "SELECT p.* FROM posts p
                 JOIN collection_bookmarks cb ON p.id = cb.post_id
                 WHERE cb.collection_id = $1
                 ORDER BY p.created_at DESC, p.id DESC
                 LIMIT $2",
                collection_id,
                fetch_limit
            ),
        }
        .unwrap_or_default();

        let has_next_page = entities.len() > first;
        let items: Vec<Post> = entities.into_iter().take(first).map(Post::from).collect();
        (items, has_next_page)
    }
}
