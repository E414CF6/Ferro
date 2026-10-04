use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::Story as StoryModel;
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::UserLoader;
use crate::infrastructure::db::postgres::Database;

use super::UserGql;

#[derive(Clone)]
pub struct StoryGql(pub StoryModel);

#[Object]
impl StoryGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn media_url(&self) -> &str {
        &self.0.media_url
    }

    async fn caption(&self) -> Option<&str> {
        self.0.caption.as_deref()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn is_expired(&self) -> bool {
        Utc::now() > self.0.expires_at
    }

    /// Story Author (batch loaded via DataLoader)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::StoryAuthorNotFound, "Story author not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::StoryAuthorNotFound, "Story author not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn views_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_story_views_count(self.0.id).await)
    }

    async fn is_viewed_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_story_viewed_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn viewers(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let auth_user = ctx.data_opt::<AuthUser>().ok_or_else(|| {
            DomainError::new(
                ErrorCode::StoryViewersAuthRequired,
                "Authentication required to view story viewers",
            )
            .extend()
        })?;

        if auth_user.user_id != self.0.author_id {
            return Err(DomainError::new(
                ErrorCode::StoryViewersUnauthorized,
                "Unauthorized: Only story authors can view viewer list",
            )
            .extend());
        }

        let db = ctx.data::<Database>()?;
        let viewers = db.get_story_viewers(self.0.id).await;
        Ok(viewers.into_iter().map(UserGql).collect())
    }
}
