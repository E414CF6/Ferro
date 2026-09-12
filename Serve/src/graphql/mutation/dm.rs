use crate::application::helpers::resolve_user_id;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::events::EventPublisher;
use crate::domain::models::TypingEvent;
use crate::domain::repositories::{DmRepository, UserRepository};
use crate::graphql::types::{DirectMessageGql, GroupConversationGql};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct DmMutation;

#[Object]
impl DmMutation {
    /// Send a direct message to another user
    async fn send_direct_message(
        &self,
        ctx: &Context<'_>,
        recipient_id: ID,
        content: String,
        sender_id: Option<ID>,
    ) -> Result<DirectMessageGql> {
        let sen_id = resolve_user_id(ctx, sender_id)?;
        let rec_id = Uuid::parse_str(&recipient_id)?;

        if sen_id == rec_id {
            return Err(DomainError::new(
                ErrorCode::DmCannotSendToSelf,
                ErrorCode::DmCannotSendToSelf.as_str(),
            )
            .extend());
        }

        let db = ctx.data::<Database>()?;

        if db.is_blocked_between(sen_id, rec_id).await {
            return Err(
                DomainError::new(ErrorCode::UserBlocked, "Cannot message blocked user").extend(),
            );
        }

        let valid_content =
            crate::domain::validation::validate_dm_content(&content).map_err(|e| e.extend())?;

        let msg = db
            .send_direct_message(sen_id, rec_id, valid_content)
            .await
            .map_err(|e| e.extend())?;

        if let Ok(broker) = ctx.data::<MessageBroker>() {
            broker.publish_dm(msg.clone());
        }

        Ok(DirectMessageGql(msg))
    }

    /// Create a group chat conversation
    async fn create_group_conversation(
        &self,
        ctx: &Context<'_>,
        title: Option<String>,
        participant_ids: Vec<ID>,
        creator_id: Option<ID>,
    ) -> Result<GroupConversationGql> {
        let cid = resolve_user_id(ctx, creator_id)?;
        let mut member_uuids = Vec::new();
        for pid_str in participant_ids {
            let pid = Uuid::parse_str(&pid_str)?;
            member_uuids.push(pid);
        }

        let db = ctx.data::<Database>()?;
        let convo = db
            .create_group_conversation(cid, title, member_uuids)
            .await
            .map_err(|e| e.extend())?;

        Ok(GroupConversationGql(convo))
    }

    /// Send a message to a group conversation
    async fn send_group_message(
        &self,
        ctx: &Context<'_>,
        conversation_id: ID,
        content: String,
        sender_id: Option<ID>,
    ) -> Result<DirectMessageGql> {
        let sid = resolve_user_id(ctx, sender_id)?;
        let cid = Uuid::parse_str(&conversation_id)?;
        let valid_content =
            crate::domain::validation::validate_dm_content(&content).map_err(|e| e.extend())?;

        let db = ctx.data::<Database>()?;
        let msg = db
            .send_conversation_message(sid, cid, valid_content)
            .await
            .map_err(|e| e.extend())?;

        if let Ok(broker) = ctx.data::<MessageBroker>() {
            broker.publish_dm(msg.clone());
        }

        Ok(DirectMessageGql(msg))
    }

    /// Send a typing indicator event
    async fn send_typing_indicator(
        &self,
        ctx: &Context<'_>,
        conversation_id: Option<ID>,
        recipient_id: Option<ID>,
        is_typing: bool,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid_uuid = if let Some(ref c) = conversation_id {
            Some(Uuid::parse_str(c)?)
        } else {
            None
        };
        let rid_uuid = if let Some(ref r) = recipient_id {
            Some(Uuid::parse_str(r)?)
        } else {
            None
        };

        if let Ok(broker) = ctx.data::<MessageBroker>() {
            broker.publish_typing(TypingEvent {
                user_id: uid,
                conversation_id: cid_uuid,
                recipient_id: rid_uuid,
                is_typing,
            });
        }

        Ok(true)
    }

    /// Edit an existing direct message
    async fn edit_direct_message(
        &self,
        ctx: &Context<'_>,
        message_id: ID,
        content: String,
        sender_id: Option<ID>,
    ) -> Result<DirectMessageGql> {
        let sid = resolve_user_id(ctx, sender_id)?;
        let mid = Uuid::parse_str(&message_id)?;
        let valid_content =
            crate::domain::validation::validate_dm_content(&content).map_err(|e| e.extend())?;

        let db = ctx.data::<Database>()?;
        let msg = db
            .edit_direct_message(mid, sid, valid_content)
            .await
            .map_err(|e| e.extend())?;

        Ok(DirectMessageGql(msg))
    }

    /// Delete a direct message
    async fn delete_direct_message(
        &self,
        ctx: &Context<'_>,
        message_id: ID,
        sender_id: Option<ID>,
    ) -> Result<bool> {
        let sid = resolve_user_id(ctx, sender_id)?;
        let mid = Uuid::parse_str(&message_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_direct_message(mid, sid)
            .await
            .map_err(|e| e.extend())
    }

    /// Mark all direct messages from a sender as read
    async fn mark_messages_as_read(
        &self,
        ctx: &Context<'_>,
        sender_id: ID,
        reader_id: Option<ID>,
    ) -> Result<bool> {
        let rdr_id = resolve_user_id(ctx, reader_id)?;
        let sen_id = Uuid::parse_str(&sender_id)?;
        let db = ctx.data::<Database>()?;
        db.mark_direct_messages_as_read(rdr_id, sen_id)
            .await
            .map_err(|e| e.extend())
    }
}
