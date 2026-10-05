use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    FollowRequest as FollowRequestModel,
    FollowRequestStatus as FollowRequestStatusModel,
    User as UserModel,
    UserList as UserListModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::{
    FollowersCountLoader, FollowingCountLoader, HasActiveStoriesLoader,
    HasPendingFollowRequestLoader, IsBlockedLoader, IsFollowingLoader,
    IsMutedLoader, UserLoader, UserPostsCountLoader,
};
use crate::infrastructure::db::postgres::Database;

use super::{
    decode_cursor, encode_cursor, BookmarkCollectionGql, CommentGql,
    PageInfoGql, PostConnectionGql, PostEdgeGql, PostGql, StoryGql,
};

#[derive(Clone)]
pub struct UserGql(pub UserModel);

#[Object]
impl UserGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn email(&self) -> &str {
        &self.0.email
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn bio(&self) -> Option<&str> {
        self.0.bio.as_deref()
    }

    async fn avatar_url(&self) -> Option<&str> {
        self.0.avatar_url.as_deref()
    }

    async fn header_image_url(&self) -> Option<&str> {
        self.0.header_image_url.as_deref()
    }

    async fn location(&self) -> Option<&str> {
        self.0.location.as_deref()
    }

    async fn website(&self) -> Option<&str> {
        self.0.website.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn is_2fa_enabled(&self) -> bool {
        self.0.is_2fa_enabled
    }

    #[graphql(name = "is2faEnabled")]
    async fn is_2fa_enabled_alias(&self) -> bool {
        self.0.is_2fa_enabled
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// User posts count (batch loaded via DataLoader)
    async fn posts_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserPostsCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_user_posts_count(self.0.id).await)
        }
    }

    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_posts_by_author(self.0.id).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn active_stories(&self, ctx: &Context<'_>) -> Result<Vec<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let stories = db.get_active_stories_for_user(self.0.id).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Whether user has active stories in last 24 hours (batch loaded via DataLoader)
    async fn has_active_stories(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(loader) = ctx.data_opt::<DataLoader<HasActiveStoriesLoader>>() {
            let has = loader.load_one(self.0.id).await?.unwrap_or(false);
            Ok(has)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.has_active_stories(self.0.id).await)
        }
    }

    async fn liked_posts(&self, ctx: &Context<'_>) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_liked_posts_by_user(self.0.id).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn saved_posts(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_saved_posts(self.0.id, limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<BookmarkCollectionGql>> {
        let db = ctx.data::<Database>()?;
        let colls = db.get_user_collections(self.0.id).await;
        Ok(colls.into_iter().map(BookmarkCollectionGql).collect())
    }

    async fn lists(&self, ctx: &Context<'_>) -> Result<Vec<UserListGql>> {
        let db = ctx.data::<Database>()?;
        let lists = db.get_user_lists(self.0.id).await;
        Ok(lists.into_iter().map(UserListGql).collect())
    }

    async fn comments(&self, ctx: &Context<'_>) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let comments = db.get_comments_by_user(self.0.id).await;
        Ok(comments.into_iter().map(CommentGql).collect())
    }

    async fn followers(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let followers = db.get_followers(self.0.id).await;
        Ok(followers.into_iter().map(UserGql).collect())
    }

    async fn following(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let following = db.get_following(self.0.id).await;
        Ok(following.into_iter().map(UserGql).collect())
    }

    /// Followers count (batch loaded via DataLoader)
    async fn followers_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<FollowersCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_followers_count(self.0.id).await)
        }
    }

    /// Following count (batch loaded via DataLoader)
    async fn following_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<FollowingCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_following_count(self.0.id).await)
        }
    }

    async fn is_me(&self, ctx: &Context<'_>) -> bool {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id == self.0.id
        } else {
            false
        }
    }

    async fn is_followed_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            if let Some(loader) = ctx.data_opt::<DataLoader<IsFollowingLoader>>() {
                let res = loader
                    .load_one((auth_user.user_id, self.0.id))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?
                    .unwrap_or(false);
                Ok(res)
            } else {
                let db = ctx.data::<Database>()?;
                Ok(db.is_following(auth_user.user_id, self.0.id).await)
            }
        } else {
            Ok(false)
        }
    }

    async fn is_blocking_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            if let Some(loader) = ctx.data_opt::<DataLoader<IsBlockedLoader>>() {
                let res = loader
                    .load_one((self.0.id, auth_user.user_id))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?
                    .unwrap_or(false);
                Ok(res)
            } else {
                let db = ctx.data::<Database>()?;
                Ok(db.is_blocking(self.0.id, auth_user.user_id).await)
            }
        } else {
            Ok(false)
        }
    }

    async fn is_blocked_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            if let Some(loader) = ctx.data_opt::<DataLoader<IsBlockedLoader>>() {
                let res = loader
                    .load_one((auth_user.user_id, self.0.id))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?
                    .unwrap_or(false);
                Ok(res)
            } else {
                let db = ctx.data::<Database>()?;
                Ok(db.is_blocking(auth_user.user_id, self.0.id).await)
            }
        } else {
            Ok(false)
        }
    }

    async fn is_muted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            if let Some(loader) = ctx.data_opt::<DataLoader<IsMutedLoader>>() {
                let res = loader
                    .load_one((auth_user.user_id, self.0.id))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?
                    .unwrap_or(false);
                Ok(res)
            } else {
                let db = ctx.data::<Database>()?;
                Ok(db.is_muting(auth_user.user_id, self.0.id).await)
            }
        } else {
            Ok(false)
        }
    }

    async fn has_pending_follow_request(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            if let Some(loader) = ctx.data_opt::<DataLoader<HasPendingFollowRequestLoader>>() {
                let res = loader
                    .load_one((auth_user.user_id, self.0.id))
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?
                    .unwrap_or(false);
                Ok(res)
            } else {
                let db = ctx.data::<Database>()?;
                Ok(db
                    .has_pending_follow_request(auth_user.user_id, self.0.id)
                    .await)
            }
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct AuthPayloadGql {
    pub token: String,
    pub user: UserGql,
}

#[Object]
impl AuthPayloadGql {
    async fn token(&self) -> &str {
        &self.token
    }

    async fn user(&self) -> &UserGql {
        &self.user
    }
}

#[derive(Clone)]
pub struct TotpSetupGql {
    pub secret: String,
    pub otpauth_uri: String,
}

#[Object]
impl TotpSetupGql {
    async fn secret(&self) -> &str {
        &self.secret
    }

    async fn otpauth_uri(&self) -> &str {
        &self.otpauth_uri
    }
}

#[derive(Clone)]
pub struct UserListGql(pub UserListModel);

#[Object]
impl UserListGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn owner(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.owner_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Owner not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let owner = db.get_user_by_id(self.0.owner_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Owner not found").extend()
            })?;
            Ok(UserGql(owner))
        }
    }

    async fn members(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let members = db.get_list_members(self.0.id).await;
        Ok(members.into_iter().map(UserGql).collect())
    }

    async fn members_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_list_members(self.0.id).await.len())
    }

    async fn feed(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db
            .get_list_feed_cursor(self.0.id, page_size, cursor_pair)
            .await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum FollowRequestStatusGql {
    Pending,
    Accepted,
    Rejected,
}

impl From<FollowRequestStatusModel> for FollowRequestStatusGql {
    fn from(s: FollowRequestStatusModel) -> Self {
        match s {
            FollowRequestStatusModel::Pending => FollowRequestStatusGql::Pending,
            FollowRequestStatusModel::Accepted => FollowRequestStatusGql::Accepted,
            FollowRequestStatusModel::Rejected => FollowRequestStatusGql::Rejected,
        }
    }
}

#[derive(Clone)]
pub struct FollowRequestGql(pub FollowRequestModel);

#[Object]
impl FollowRequestGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn requester(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(self.0.requester_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Requester not found").extend()
            })?;
        Ok(UserGql(user))
    }

    async fn target(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db.get_user_by_id(self.0.target_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, "Target user not found").extend()
        })?;
        Ok(UserGql(user))
    }

    async fn status(&self) -> FollowRequestStatusGql {
        FollowRequestStatusGql::from(self.0.status)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}
