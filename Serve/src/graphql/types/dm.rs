use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    Conversation as ConversationModel, ConversationSummary as ConversationSummaryModel,
    DirectMessage as DirectMessageModel, TypingEvent as TypingEventModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::UserLoader;
use crate::infrastructure::db::postgres::Database;

use super::{PageInfoGql, UserGql};

#[derive(Clone)]
pub struct DirectMessageGql(pub DirectMessageModel);

#[Object]
impl DirectMessageGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn conversation_id(&self) -> Option<ID> {
        self.0.conversation_id.map(|id| ID(id.to_string()))
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn is_read(&self) -> bool {
        self.0.is_read
    }

    async fn is_edited(&self) -> bool {
        self.0.is_edited
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// Message Sender (batch loaded via DataLoader)
    async fn sender(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.sender_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::DmSenderNotFound, "Sender user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.sender_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::DmSenderNotFound, "Sender user not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    /// Message Recipient (batch loaded via DataLoader)
    async fn recipient(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.recipient_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::DmRecipientNotFound, "Recipient user not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db
                .get_user_by_id(self.0.recipient_id)
                .await
                .ok_or_else(|| {
                    DomainError::new(ErrorCode::DmRecipientNotFound, "Recipient user not found")
                        .extend()
                })?;
            Ok(UserGql(user))
        }
    }

    async fn is_mine(&self, ctx: &Context<'_>) -> bool {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id == self.0.sender_id
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct DirectMessageEdgeGql {
    pub cursor: String,
    pub node: DirectMessageGql,
}

#[Object]
impl DirectMessageEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &DirectMessageGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct DirectMessageConnectionGql {
    pub edges: Vec<DirectMessageEdgeGql>,
    pub page_info: PageInfoGql,
}

#[Object]
impl DirectMessageConnectionGql {
    async fn edges(&self) -> &[DirectMessageEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }
}

#[derive(Clone)]
pub struct ConversationGql(pub ConversationSummaryModel);

#[Object]
impl ConversationGql {
    async fn other_user(&self) -> UserGql {
        UserGql(self.0.other_user.clone())
    }

    async fn last_message(&self) -> DirectMessageGql {
        DirectMessageGql(self.0.last_message.clone())
    }

    async fn unread_count(&self) -> i64 {
        self.0.unread_count
    }
}

#[derive(Clone)]
pub struct GroupConversationGql(pub ConversationModel);

#[Object]
impl GroupConversationGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn is_group(&self) -> bool {
        self.0.is_group
    }

    async fn title(&self) -> Option<&str> {
        self.0.title.as_deref()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    async fn creator(&self, ctx: &Context<'_>) -> Result<Option<UserGql>> {
        let Some(cid) = self.0.created_by else {
            return Ok(None);
        };
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_id(cid).await.map(UserGql))
    }

    async fn participants(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.get_conversation_participants(self.0.id).await;
        Ok(users.into_iter().map(UserGql).collect())
    }

    async fn messages(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DirectMessageGql>> {
        let db = ctx.data::<Database>()?;
        let msgs = db.get_conversation_messages(self.0.id, limit, offset).await;
        Ok(msgs.into_iter().map(DirectMessageGql).collect())
    }
}

#[derive(Clone, Debug)]
pub struct TypingEventGql(pub TypingEventModel);

#[Object]
impl TypingEventGql {
    async fn user_id(&self) -> ID {
        ID(self.0.user_id.to_string())
    }

    async fn conversation_id(&self) -> Option<ID> {
        self.0.conversation_id.map(|id| ID(id.to_string()))
    }

    async fn recipient_id(&self) -> Option<ID> {
        self.0.recipient_id.map(|id| ID(id.to_string()))
    }

    async fn is_typing(&self) -> bool {
        self.0.is_typing
    }
}
