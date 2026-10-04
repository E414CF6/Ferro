use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    Post as PostModel, PostAnalytics as PostAnalyticsModel,
    PostAudience as PostAudienceModel, PostMedia as PostMediaModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::{
    CommentLoader, PostLikesCountLoader, PostLoader, PostMediaLoader,
    PostPollLoader, PostRepostsCountLoader, UserLoader,
};
use crate::infrastructure::db::postgres::Database;

use super::{
    decode_cursor, encode_cursor, CommentConnectionGql, CommentEdgeGql, CommentGql,
    MediaTypeGql, PageInfoGql, PollGql, UserGql,
};

#[derive(Clone)]
pub struct PostMediaGql(pub PostMediaModel);

#[Object]
impl PostMediaGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn media_url(&self) -> &str {
        &self.0.media_url
    }

    async fn media_type(&self) -> MediaTypeGql {
        MediaTypeGql::from(self.0.media_type)
    }

    async fn alt_text(&self) -> Option<&str> {
        self.0.alt_text.as_deref()
    }

    async fn sort_order(&self) -> i32 {
        self.0.sort_order
    }

    async fn width(&self) -> Option<i32> {
        self.0.width
    }

    async fn height(&self) -> Option<i32> {
        self.0.height
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PostAudienceGql {
    Public,
    FollowersOnly,
    CloseFriends,
}

impl From<PostAudienceModel> for PostAudienceGql {
    fn from(a: PostAudienceModel) -> Self {
        match a {
            PostAudienceModel::Public => PostAudienceGql::Public,
            PostAudienceModel::FollowersOnly => PostAudienceGql::FollowersOnly,
            PostAudienceModel::CloseFriends => PostAudienceGql::CloseFriends,
        }
    }
}

impl From<PostAudienceGql> for PostAudienceModel {
    fn from(a: PostAudienceGql) -> Self {
        match a {
            PostAudienceGql::Public => PostAudienceModel::Public,
            PostAudienceGql::FollowersOnly => PostAudienceModel::FollowersOnly,
            PostAudienceGql::CloseFriends => PostAudienceModel::CloseFriends,
        }
    }
}

#[derive(Clone)]
pub struct PostGql(pub PostModel);

#[Object]
impl PostGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn audience(&self) -> PostAudienceGql {
        PostAudienceGql::from(self.0.audience)
    }

    async fn views_count(&self) -> i64 {
        self.0.views_count
    }

    async fn quote_post_id(&self) -> Option<ID> {
        self.0.quote_post_id.map(|id| ID(id.to_string()))
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// Post Author (batch loaded via DataLoader to eliminate N+1)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::PostAuthorNotFound, "Post author not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::PostAuthorNotFound, "Post author not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    /// Media Attachments (batch loaded via DataLoader)
    async fn media(&self, ctx: &Context<'_>) -> Result<Vec<PostMediaGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostMediaLoader>>() {
            let media_list = loader.load_one(self.0.id).await?.unwrap_or_default();
            Ok(media_list.into_iter().map(PostMediaGql).collect())
        } else {
            let db = ctx.data::<Database>()?;
            let media_list = db.get_post_media(self.0.id).await;
            Ok(media_list.into_iter().map(PostMediaGql).collect())
        }
    }

    /// Attached Poll (if any)
    async fn poll(&self, ctx: &Context<'_>) -> Result<Option<PollGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostPollLoader>>() {
            let poll_opt = loader.load_one(self.0.id).await?.flatten();
            Ok(poll_opt.map(PollGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_poll_by_post_id(self.0.id).await.map(PollGql))
        }
    }

    /// Post Analytics & Insights
    async fn analytics(&self, ctx: &Context<'_>) -> Result<PostAnalyticsGql> {
        let db = ctx.data::<Database>()?;
        let an = db
            .get_post_analytics(self.0.id)
            .await
            .map_err(|e| e.extend())?;
        Ok(PostAnalyticsGql(an))
    }

    /// Quoted Post (if this is a quote post, batch loaded via PostLoader)
    async fn quote_post(&self, ctx: &Context<'_>) -> Result<Option<PostGql>> {
        let Some(qid) = self.0.quote_post_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
            let post_opt = loader.load_one(qid).await?;
            Ok(post_opt.map(PostGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_post_by_id(qid).await.map(PostGql))
        }
    }

    /// Pinned Comment by Author (batch loaded via CommentLoader)
    async fn pinned_comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(cid) = self.0.pinned_comment_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
            let comment_opt = loader.load_one(cid).await?;
            Ok(comment_opt.map(CommentGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_by_id(cid).await.map(CommentGql))
        }
    }

    /// Likes Count (batch loaded via DataLoader)
    async fn likes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLikesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_likes_count(self.0.id).await)
        }
    }

    /// Reposts Count (batch loaded via DataLoader)
    async fn reposts_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostRepostsCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_reposts_count(self.0.id).await)
        }
    }

    async fn is_liked_by(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        let uid = if let Some(uid_str) = user_id {
            Uuid::parse_str(&uid_str)?
        } else if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id
        } else {
            return Ok(false);
        };

        Ok(db.is_post_liked_by(self.0.id, uid).await)
    }

    async fn is_reposted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_post_reposted_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn is_saved_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_post_saved_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn hashtags(&self) -> Vec<String> {
        self.0
            .content
            .split_whitespace()
            .filter_map(|w| {
                if let Some(tag) = w.strip_prefix('#') {
                    let clean: String = tag
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !clean.is_empty() { Some(clean) } else { None }
                } else {
                    None
                }
            })
            .collect()
    }

    async fn comments(
        &self,
        ctx: &Context<'_>,
        top_level_only: Option<bool>,
    ) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let comments = if top_level_only.unwrap_or(false) {
            db.get_top_level_comments_for_post(self.0.id).await
        } else {
            db.get_comments_for_post(self.0.id).await
        };
        Ok(comments.into_iter().map(CommentGql).collect())
    }

    /// Cursor-paginated comments on this post
    async fn comments_connection(
        &self,
        ctx: &Context<'_>,
        top_level_only: Option<bool>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<CommentConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let is_top = top_level_only.unwrap_or(true);
        let (comments, has_next_page) = db
            .get_comments_cursor(self.0.id, is_top, page_size, cursor_pair)
            .await;
        let start_cursor = comments.first().map(|c| encode_cursor(c.created_at, c.id));
        let end_cursor = comments.last().map(|c| encode_cursor(c.created_at, c.id));

        let edges = comments
            .into_iter()
            .map(|c| {
                let cursor = encode_cursor(c.created_at, c.id);
                CommentEdgeGql {
                    cursor,
                    node: CommentGql(c),
                }
            })
            .collect();

        Ok(CommentConnectionGql {
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

#[derive(Clone)]
pub struct PostAnalyticsGql(pub PostAnalyticsModel);

#[Object]
impl PostAnalyticsGql {
    async fn post_id(&self) -> ID {
        ID(self.0.post_id.to_string())
    }

    async fn views_count(&self) -> i64 {
        self.0.views_count
    }

    async fn likes_count(&self) -> usize {
        self.0.likes_count
    }

    async fn reposts_count(&self) -> usize {
        self.0.reposts_count
    }

    async fn comments_count(&self) -> usize {
        self.0.comments_count
    }

    async fn engagement_rate(&self) -> f64 {
        self.0.engagement_rate
    }
}

#[derive(Clone, Debug)]
pub struct HashtagTrendGql {
    pub tag: String,
    pub count: usize,
}

#[Object]
impl HashtagTrendGql {
    async fn tag(&self) -> &str {
        &self.tag
    }

    async fn count(&self) -> usize {
        self.count
    }
}

#[derive(Clone)]
pub struct PostEdgeGql {
    pub cursor: String,
    pub node: PostGql,
}

#[Object]
impl PostEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &PostGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct PostConnectionGql {
    pub edges: Vec<PostEdgeGql>,
    pub page_info: PageInfoGql,
    pub total_count: Option<usize>,
}

#[Object]
impl PostConnectionGql {
    async fn edges(&self) -> &[PostEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn total_count(&self) -> Option<usize> {
        self.total_count
    }
}
