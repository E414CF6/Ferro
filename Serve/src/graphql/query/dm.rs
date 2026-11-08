use crate::application::AppServices;
use crate::application::helpers::require_auth;
use crate::graphql::types::{
    ConversationGql, DirectMessageConnectionGql, DirectMessageEdgeGql, DirectMessageGql,
    GroupConversationGql, PageInfoGql, decode_cursor, encode_cursor,
};
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct DmQuery;

#[Object]
impl DmQuery {
    /// Get conversation summaries for current user
    async fn conversations(&self, ctx: &Context<'_>) -> Result<Vec<ConversationGql>> {
        let uid = require_auth(ctx)?;
        let services = ctx.data::<AppServices>()?;
        let convos = services
            .dm
            .get_conversations(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(convos.into_iter().map(ConversationGql).collect())
    }

    /// Get direct messages with another user with offset pagination
    async fn direct_messages(
        &self,
        ctx: &Context<'_>,
        other_user_id: ID,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DirectMessageGql>> {
        let uid = require_auth(ctx)?;
        let other_id = Uuid::parse_str(&other_user_id)?;
        let services = ctx.data::<AppServices>()?;
        let msgs = services
            .dm
            .get_direct_messages(uid, other_id, limit, offset)
            .await
            .map_err(|e| e.extend())?;
        Ok(msgs.into_iter().map(DirectMessageGql).collect())
    }

    /// Get direct messages with another user via cursor pagination
    async fn direct_messages_connection(
        &self,
        ctx: &Context<'_>,
        other_user_id: ID,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<DirectMessageConnectionGql> {
        let uid = require_auth(ctx)?;
        let other_id = Uuid::parse_str(&other_user_id)?;
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (msgs, has_next_page) = services
            .dm
            .get_direct_messages_cursor(uid, other_id, page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
        let start_cursor = msgs.first().map(|m| encode_cursor(m.created_at, m.id));
        let end_cursor = msgs.last().map(|m| encode_cursor(m.created_at, m.id));

        let edges = msgs
            .into_iter()
            .map(|msg| {
                let cursor = encode_cursor(msg.created_at, msg.id);
                DirectMessageEdgeGql {
                    cursor,
                    node: DirectMessageGql(msg),
                }
            })
            .collect();

        Ok(DirectMessageConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
        })
    }

    /// Get group conversations the current user belongs to
    async fn group_conversations(&self, ctx: &Context<'_>) -> Result<Vec<GroupConversationGql>> {
        let uid = require_auth(ctx)?;
        let services = ctx.data::<AppServices>()?;
        let convos = services
            .dm
            .get_user_group_conversations(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(convos.into_iter().map(GroupConversationGql).collect())
    }

    /// Get single group conversation by ID
    async fn group_conversation(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<GroupConversationGql>> {
        let _ = require_auth(ctx)?;
        let cid = Uuid::parse_str(&id)?;
        let services = ctx.data::<AppServices>()?;
        let convo = services
            .dm
            .get_conversation_by_id(cid)
            .await
            .map_err(|e| e.extend())?;
        Ok(convo.map(GroupConversationGql))
    }

    /// Get total unread direct message count for current user
    async fn unread_dm_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let uid = require_auth(ctx)?;
        let services = ctx.data::<AppServices>()?;
        Ok(services.dm.get_unread_dm_count(uid).await)
    }
}
