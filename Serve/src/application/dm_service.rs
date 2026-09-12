#![allow(dead_code)]
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Conversation, ConversationSummary, DirectMessage, User};
use crate::domain::repositories::DmRepository;
use crate::domain::validation::validate_dm_content;
use crate::infrastructure::db::database::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating Direct Messages, Group Chats, and Conversations
#[derive(Clone)]
pub struct DmService<R: DmRepository = Database> {
    repo: Arc<R>,
}

impl<R: DmRepository> DmService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn send_direct_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError> {
        if sender_id == recipient_id {
            return Err(DomainError::new(
                ErrorCode::DmCannotSendToSelf,
                "Cannot send direct message to yourself",
            ));
        }
        validate_dm_content(&content)?;
        self.repo
            .send_direct_message(sender_id, recipient_id, content)
            .await
    }

    pub async fn get_direct_messages_cursor(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool) {
        self.repo
            .get_direct_messages_cursor(user1_id, user2_id, first, after)
            .await
    }

    pub async fn get_conversations(&self, user_id: Uuid) -> Vec<ConversationSummary> {
        self.repo.get_conversations(user_id).await
    }

    pub async fn mark_direct_messages_as_read(
        &self,
        reader_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo
            .mark_direct_messages_as_read(reader_id, sender_id)
            .await
    }

    pub async fn get_unread_dm_count(&self, user_id: Uuid) -> usize {
        self.repo.get_unread_dm_count(user_id).await
    }

    pub async fn create_group_conversation(
        &self,
        creator_id: Uuid,
        title: Option<String>,
        participant_ids: Vec<Uuid>,
    ) -> Result<Conversation, DomainError> {
        self.repo
            .create_group_conversation(creator_id, title, participant_ids)
            .await
    }

    pub async fn get_conversation_by_id(&self, conversation_id: Uuid) -> Option<Conversation> {
        self.repo.get_conversation_by_id(conversation_id).await
    }

    pub async fn get_conversation_participants(&self, conversation_id: Uuid) -> Vec<User> {
        self.repo
            .get_conversation_participants(conversation_id)
            .await
    }

    pub async fn send_conversation_message(
        &self,
        sender_id: Uuid,
        conversation_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError> {
        validate_dm_content(&content)?;
        self.repo
            .send_conversation_message(sender_id, conversation_id, content)
            .await
    }

    pub async fn get_conversation_messages_cursor(
        &self,
        conversation_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool) {
        self.repo
            .get_conversation_messages_cursor(conversation_id, first, after)
            .await
    }

    pub async fn edit_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
        new_content: String,
    ) -> Result<DirectMessage, DomainError> {
        validate_dm_content(&new_content)?;
        self.repo
            .edit_direct_message(message_id, sender_id, new_content)
            .await
    }

    pub async fn delete_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.repo.delete_direct_message(message_id, sender_id).await
    }

    pub async fn get_user_group_conversations(&self, user_id: Uuid) -> Vec<Conversation> {
        self.repo.get_user_group_conversations(user_id).await
    }
}
