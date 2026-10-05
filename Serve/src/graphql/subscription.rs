use crate::application::helpers::require_auth;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::events::EventPublisher;
use crate::graphql::types::{DirectMessageGql, NotificationGql, TypingEventGql};
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::{Context, ErrorExtensions, ID, Result, Subscription};
use futures_util::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Real-time subscription to new direct messages addressed to the specified recipient user ID.
    /// Enforces authentication and authorization: only the intended recipient can subscribe.
    async fn direct_message_received(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> Result<impl Stream<Item = DirectMessageGql>> {
        let auth_uid = require_auth(ctx)?;
        let target_uid = Uuid::parse_str(&user_id)?;
        if auth_uid != target_uid {
            return Err(DomainError::new(
                ErrorCode::ErrorForbidden,
                ErrorCode::ErrorForbidden.as_str(),
            )
            .extend());
        }

        let broker = ctx.data::<MessageBroker>()?;
        let rx = broker.subscribe_dm();

        let stream = BroadcastStream::new(rx).filter_map(move |item| match item {
            Ok(msg) => {
                if msg.recipient_id == target_uid {
                    Some(DirectMessageGql(msg))
                } else {
                    None
                }
            }
            Err(BroadcastStreamRecvError::Lagged(missed)) => {
                tracing::warn!(
                    target: "serve::subscription",
                    user_id = %target_uid,
                    missed_count = missed,
                    "DM subscription lagged behind; dropped missed messages"
                );
                None
            }
        });

        Ok(stream)
    }

    /// Real-time subscription to notifications addressed to the specified user ID.
    /// Enforces authentication and authorization: only the intended recipient can subscribe.
    async fn notification_received(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> Result<impl Stream<Item = NotificationGql>> {
        let auth_uid = require_auth(ctx)?;
        let target_uid = Uuid::parse_str(&user_id)?;
        if auth_uid != target_uid {
            return Err(DomainError::new(
                ErrorCode::ErrorForbidden,
                ErrorCode::ErrorForbidden.as_str(),
            )
            .extend());
        }

        let broker = ctx.data::<MessageBroker>()?;
        let rx = broker.subscribe_notification();

        let stream = BroadcastStream::new(rx).filter_map(move |item| match item {
            Ok(notif) => {
                if notif.recipient_id == target_uid {
                    Some(NotificationGql(notif))
                } else {
                    None
                }
            }
            Err(BroadcastStreamRecvError::Lagged(missed)) => {
                tracing::warn!(
                    target: "serve::subscription",
                    user_id = %target_uid,
                    missed_count = missed,
                    "Notification subscription lagged behind; dropped missed notifications"
                );
                None
            }
        });

        Ok(stream)
    }

    /// Real-time subscription to typing indicator events in a conversation or direct chat.
    /// Enforces authentication. If recipient_id is provided, verifies that caller matches recipient.
    async fn typing_status(
        &self,
        ctx: &Context<'_>,
        conversation_id: Option<ID>,
        recipient_id: Option<ID>,
    ) -> Result<impl Stream<Item = TypingEventGql>> {
        let auth_uid = require_auth(ctx)?;

        if conversation_id.is_none() && recipient_id.is_none() {
            return Err(DomainError::new(
                ErrorCode::ErrorBadRequest,
                ErrorCode::ErrorBadRequest.as_str(),
            )
            .extend());
        }

        let target_cid = if let Some(ref c) = conversation_id {
            Some(Uuid::parse_str(c)?)
        } else {
            None
        };
        let target_rid = if let Some(ref r) = recipient_id {
            let rid = Uuid::parse_str(r)?;
            if rid != auth_uid {
                return Err(DomainError::new(
                    ErrorCode::ErrorForbidden,
                    ErrorCode::ErrorForbidden.as_str(),
                )
                .extend());
            }
            Some(rid)
        } else {
            None
        };

        let broker = ctx.data::<MessageBroker>()?;
        let rx = broker.subscribe_typing();

        let stream = BroadcastStream::new(rx).filter_map(move |item| match item {
            Ok(event) => {
                if let Some(cid) = target_cid {
                    if event.conversation_id == Some(cid) {
                        return Some(TypingEventGql(event));
                    }
                }
                if let Some(rid) = target_rid {
                    if event.recipient_id == Some(rid) {
                        return Some(TypingEventGql(event));
                    }
                }
                None
            }
            Err(BroadcastStreamRecvError::Lagged(missed)) => {
                tracing::debug!(
                    target: "serve::subscription",
                    missed_count = missed,
                    "Typing indicator subscription lagged behind; dropped events"
                );
                None
            }
        });

        Ok(stream)
    }
}
