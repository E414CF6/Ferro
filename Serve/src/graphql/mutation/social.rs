use crate::application::helpers::resolve_user_id;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::NotificationType;
use crate::domain::repositories::{NotificationRepository, PostRepository, UserRepository};
use crate::graphql::types::{UserGql, UserListGql};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct SocialMutation;

#[Object]
impl SocialMutation {
    /// Block a user
    async fn block_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        blocker_id: Option<ID>,
    ) -> Result<bool> {
        let bid = resolve_user_id(ctx, blocker_id)?;
        let target_id = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.block_user(bid, target_id).await.map_err(|e| e.extend())
    }

    /// Unblock a user
    async fn unblock_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        blocker_id: Option<ID>,
    ) -> Result<bool> {
        let bid = resolve_user_id(ctx, blocker_id)?;
        let target_id = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.unblock_user(bid, target_id)
            .await
            .map_err(|e| e.extend())
    }

    /// Mute a user
    async fn mute_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        muter_id: Option<ID>,
    ) -> Result<bool> {
        let mid = resolve_user_id(ctx, muter_id)?;
        let target_id = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.mute_user(mid, target_id).await.map_err(|e| e.extend())
    }

    /// Unmute a user
    async fn unmute_user(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        muter_id: Option<ID>,
    ) -> Result<bool> {
        let mid = resolve_user_id(ctx, muter_id)?;
        let target_id = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.unmute_user(mid, target_id).await.map_err(|e| e.extend())
    }

    /// Accept a pending follow request
    async fn accept_follow_request(
        &self,
        ctx: &Context<'_>,
        requester_id: ID,
        target_id: Option<ID>,
    ) -> Result<UserGql> {
        let tid = resolve_user_id(ctx, target_id)?;
        let rid = Uuid::parse_str(&requester_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .accept_follow_request(tid, rid)
            .await
            .map_err(|e| e.extend())?;

        // Send follow accepted notification
        if let Ok(broker) = ctx.data::<MessageBroker>() {
            if let Ok(notif) = db
                .create_notification(rid, tid, NotificationType::FollowAccepted, None)
                .await
            {
                broker.publish_notification(notif);
            }
        }

        Ok(UserGql(user))
    }

    /// Reject a pending follow request
    async fn reject_follow_request(
        &self,
        ctx: &Context<'_>,
        requester_id: ID,
        target_id: Option<ID>,
    ) -> Result<bool> {
        let tid = resolve_user_id(ctx, target_id)?;
        let rid = Uuid::parse_str(&requester_id)?;
        let db = ctx.data::<Database>()?;
        db.reject_follow_request(tid, rid)
            .await
            .map_err(|e| e.extend())
    }

    /// Follow another user (creates follow request if account is private)
    async fn follow_user(
        &self,
        ctx: &Context<'_>,
        followee_id: ID,
        follower_id: Option<ID>,
    ) -> Result<UserGql> {
        let fr_id = resolve_user_id(ctx, follower_id)?;
        let fe_id = Uuid::parse_str(&followee_id)?;

        if fr_id == fe_id {
            return Err(DomainError::new(
                ErrorCode::UserCannotFollowSelf,
                ErrorCode::UserCannotFollowSelf.as_str(),
            )
            .extend());
        }

        let db = ctx.data::<Database>()?;
        let target_user = db.get_user_by_id(fe_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, "Target user not found").extend()
        })?;

        if target_user.is_private {
            // Create follow request for private account
            let _ = db
                .create_follow_request(fr_id, fe_id)
                .await
                .map_err(|e| e.extend())?;
            if let Ok(broker) = ctx.data::<MessageBroker>() {
                if let Ok(notif) = db
                    .create_notification(fe_id, fr_id, NotificationType::FollowRequest, None)
                    .await
                {
                    broker.publish_notification(notif);
                }
            }
            Ok(UserGql(target_user))
        } else {
            // Direct follow for public account
            let followee = db.follow_user(fr_id, fe_id).await.map_err(|e| e.extend())?;
            if let Ok(notif) = db
                .create_notification(fe_id, fr_id, NotificationType::Follow, None)
                .await
            {
                if let Ok(broker) = ctx.data::<MessageBroker>() {
                    broker.publish_notification(notif);
                }
            }
            Ok(UserGql(followee))
        }
    }

    /// Unfollow a user
    async fn unfollow_user(
        &self,
        ctx: &Context<'_>,
        followee_id: ID,
        follower_id: Option<ID>,
    ) -> Result<UserGql> {
        let fr_id = resolve_user_id(ctx, follower_id)?;
        let fe_id = Uuid::parse_str(&followee_id)?;
        let db = ctx.data::<Database>()?;
        let followee = db
            .unfollow_user(fr_id, fe_id)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(followee))
    }

    /// Create a custom user list
    async fn create_user_list(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        is_private: Option<bool>,
        member_ids: Option<Vec<ID>>,
        owner_id: Option<ID>,
    ) -> Result<UserListGql> {
        let oid = resolve_user_id(ctx, owner_id)?;
        let mut members = Vec::new();
        if let Some(mids) = member_ids {
            for id_str in mids {
                members.push(Uuid::parse_str(&id_str)?);
            }
        }

        let db = ctx.data::<Database>()?;
        let list = db
            .create_user_list(oid, name, description, is_private.unwrap_or(false), members)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserListGql(list))
    }

    /// Update user list details
    async fn update_user_list(
        &self,
        ctx: &Context<'_>,
        list_id: ID,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
        owner_id: Option<ID>,
    ) -> Result<UserListGql> {
        let oid = resolve_user_id(ctx, owner_id)?;
        let lid = Uuid::parse_str(&list_id)?;
        let db = ctx.data::<Database>()?;
        let list = db
            .update_user_list(lid, oid, name, description, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserListGql(list))
    }

    /// Delete a user list
    async fn delete_user_list(
        &self,
        ctx: &Context<'_>,
        list_id: ID,
        owner_id: Option<ID>,
    ) -> Result<bool> {
        let oid = resolve_user_id(ctx, owner_id)?;
        let lid = Uuid::parse_str(&list_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_user_list(lid, oid).await.map_err(|e| e.extend())
    }

    /// Add a user to a list
    async fn add_user_to_list(
        &self,
        ctx: &Context<'_>,
        list_id: ID,
        user_id: ID,
        owner_id: Option<ID>,
    ) -> Result<bool> {
        let oid = resolve_user_id(ctx, owner_id)?;
        let lid = Uuid::parse_str(&list_id)?;
        let uid = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.add_user_to_list(lid, oid, uid)
            .await
            .map_err(|e| e.extend())
    }

    /// Remove a user from a list
    async fn remove_user_from_list(
        &self,
        ctx: &Context<'_>,
        list_id: ID,
        user_id: ID,
        owner_id: Option<ID>,
    ) -> Result<bool> {
        let oid = resolve_user_id(ctx, owner_id)?;
        let lid = Uuid::parse_str(&list_id)?;
        let uid = Uuid::parse_str(&user_id)?;
        let db = ctx.data::<Database>()?;
        db.remove_user_from_list(lid, oid, uid)
            .await
            .map_err(|e| e.extend())
    }
}
