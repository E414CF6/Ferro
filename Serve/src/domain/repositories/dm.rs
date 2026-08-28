use crate::domain::errors::DomainError;
use crate::domain::models::{Conversation, ConversationSummary, DirectMessage, User};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait DmRepository: Send + Sync {
    async fn send_direct_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError>;

    async fn get_direct_messages(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<DirectMessage>;

    async fn get_direct_messages_cursor(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool);

    async fn get_conversations(&self, user_id: Uuid) -> Vec<ConversationSummary>;

    async fn mark_direct_messages_as_read(
        &self,
        reader_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError>;

    async fn get_unread_dm_count(&self, user_id: Uuid) -> usize;

    // Advanced Direct Messaging: Group chats, message edits & deletes
    async fn create_group_conversation(
        &self,
        creator_id: Uuid,
        title: Option<String>,
        participant_ids: Vec<Uuid>,
    ) -> Result<Conversation, DomainError>;

    async fn get_conversation_by_id(&self, conversation_id: Uuid) -> Option<Conversation>;

    async fn get_conversation_participants(&self, conversation_id: Uuid) -> Vec<User>;

    async fn send_conversation_message(
        &self,
        sender_id: Uuid,
        conversation_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError>;

    async fn get_conversation_messages(
        &self,
        conversation_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<DirectMessage>;

    async fn get_conversation_messages_cursor(
        &self,
        conversation_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool);

    async fn edit_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
        new_content: String,
    ) -> Result<DirectMessage, DomainError>;

    async fn delete_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError>;

    async fn get_user_group_conversations(&self, user_id: Uuid) -> Vec<Conversation>;
}
