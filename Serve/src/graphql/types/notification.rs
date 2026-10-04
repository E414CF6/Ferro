use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    Notification as NotificationModel, NotificationType as NotificationTypeModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::db::loaders::{CommentLoader, PostLoader, UserLoader};
use crate::infrastructure::db::postgres::Database;

use super::{CommentGql, PageInfoGql, PostGql, UserGql};

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotificationTypeGql {
    LikePost,
    CommentPost,
    Follow,
    ReplyComment,
    Repost,
    Quote,
    Mention,
    FollowRequest,
    FollowAccepted,
    LikeComment,
    PollEnded,
}

impl From<NotificationTypeModel> for NotificationTypeGql {
    fn from(t: NotificationTypeModel) -> Self {
        match t {
            NotificationTypeModel::LikePost => NotificationTypeGql::LikePost,
            NotificationTypeModel::CommentPost => NotificationTypeGql::CommentPost,
            NotificationTypeModel::Follow => NotificationTypeGql::Follow,
            NotificationTypeModel::ReplyComment => NotificationTypeGql::ReplyComment,
            NotificationTypeModel::Repost => NotificationTypeGql::Repost,
            NotificationTypeModel::Quote => NotificationTypeGql::Quote,
            NotificationTypeModel::Mention => NotificationTypeGql::Mention,
            NotificationTypeModel::FollowRequest => NotificationTypeGql::FollowRequest,
            NotificationTypeModel::FollowAccepted => NotificationTypeGql::FollowAccepted,
            NotificationTypeModel::LikeComment => NotificationTypeGql::LikeComment,
            NotificationTypeModel::PollEnded => NotificationTypeGql::PollEnded,
        }
    }
}

#[derive(Clone)]
pub struct NotificationGql(pub NotificationModel);

#[Object]
impl NotificationGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn r#type(&self) -> NotificationTypeGql {
        NotificationTypeGql::from(self.0.notification_type)
    }

    async fn notification_type(&self) -> NotificationTypeGql {
        NotificationTypeGql::from(self.0.notification_type)
    }

    async fn actor(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.actor_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Actor user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.actor_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Actor user not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn recipient(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.recipient_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Recipient user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db
                .get_user_by_id(self.0.recipient_id)
                .await
                .ok_or_else(|| {
                    DomainError::new(ErrorCode::UserNotFound, "Recipient user not found").extend()
                })?;
            Ok(UserGql(user))
        }
    }

    async fn entity_id(&self) -> Option<ID> {
        self.0.entity_id.map(|id| ID(id.to_string()))
    }

    async fn is_read(&self) -> bool {
        self.0.is_read
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn post(&self, ctx: &Context<'_>) -> Result<Option<PostGql>> {
        let Some(eid) = self.0.entity_id else {
            return Ok(None);
        };
        match self.0.notification_type {
            NotificationTypeModel::LikePost
            | NotificationTypeModel::Repost
            | NotificationTypeModel::Quote
            | NotificationTypeModel::Mention
            | NotificationTypeModel::PollEnded => {
                if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
                    let post_opt = loader.load_one(eid).await?;
                    Ok(post_opt.map(PostGql))
                } else {
                    let db = ctx.data::<Database>()?;
                    Ok(db.get_post_by_id(eid).await.map(PostGql))
                }
            }
            NotificationTypeModel::CommentPost
            | NotificationTypeModel::ReplyComment
            | NotificationTypeModel::LikeComment => {
                let comment_opt = if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>()
                {
                    loader.load_one(eid).await?
                } else {
                    let db = ctx.data::<Database>()?;
                    db.get_comment_by_id(eid).await
                };

                if let Some(comment) = comment_opt {
                    if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
                        let post_opt = loader.load_one(comment.post_id).await?;
                        Ok(post_opt.map(PostGql))
                    } else {
                        let db = ctx.data::<Database>()?;
                        Ok(db.get_post_by_id(comment.post_id).await.map(PostGql))
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    async fn comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(eid) = self.0.entity_id else {
            return Ok(None);
        };
        match self.0.notification_type {
            NotificationTypeModel::CommentPost
            | NotificationTypeModel::ReplyComment
            | NotificationTypeModel::LikeComment => {
                if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
                    let comment_opt = loader.load_one(eid).await?;
                    Ok(comment_opt.map(CommentGql))
                } else {
                    let db = ctx.data::<Database>()?;
                    Ok(db.get_comment_by_id(eid).await.map(CommentGql))
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct NotificationEdgeGql {
    pub cursor: String,
    pub node: NotificationGql,
}

#[Object]
impl NotificationEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &NotificationGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct NotificationConnectionGql {
    pub edges: Vec<NotificationEdgeGql>,
    pub page_info: PageInfoGql,
    pub unread_count: usize,
}

#[Object]
impl NotificationConnectionGql {
    async fn edges(&self) -> &[NotificationEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn unread_count(&self) -> usize {
        self.unread_count
    }
}
