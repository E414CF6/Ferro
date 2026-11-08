#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{FollowRequest, User, UserList};
use crate::domain::repositories::{PostRepository, UserRepository};
use crate::infrastructure::db::postgres::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating User profiles, Social graph (follow/unfollow),
/// Trust & Safety (block/unblock, mute/unmute), Follow requests, and User lists
#[derive(Clone)]
pub struct SocialService {
    db: Arc<Database>,
}

impl SocialService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, DomainError> {
        self.db.get_user_by_id(user_id).await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        self.db.get_user_by_username(username).await
    }

    pub async fn get_users(&self) -> Result<Vec<User>, DomainError> {
        self.db.get_users().await
    }

    pub async fn update_user_profile(
        &self,
        user_id: Uuid,
        username: Option<String>,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<User, DomainError> {
        self.db
            .update_user_profile(
                user_id,
                username,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await
    }

    pub async fn update_user_privacy(
        &self,
        user_id: Uuid,
        is_private: bool,
    ) -> Result<User, DomainError> {
        self.db.update_user_privacy(user_id, is_private).await
    }

    pub async fn set_totp_secret(
        &self,
        user_id: Uuid,
        secret: String,
    ) -> Result<bool, DomainError> {
        self.db.set_totp_secret(user_id, secret).await
    }

    pub async fn enable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError> {
        self.db.enable_2fa(user_id).await
    }

    pub async fn disable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError> {
        self.db.disable_2fa(user_id).await
    }

    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<User, DomainError> {
        let followee = self.db.get_user_by_id(followee_id).await?.ok_or_else(|| {
            DomainError::new(
                crate::domain::errors::ErrorCode::UserNotFound,
                "User not found",
            )
        })?;

        if followee.is_private {
            let _ = self
                .db
                .create_follow_request(follower_id, followee_id)
                .await?;
            Ok(followee)
        } else {
            self.db.follow_user(follower_id, followee_id).await
        }
    }

    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<User, DomainError> {
        self.db.unfollow_user(follower_id, followee_id).await
    }

    pub async fn get_followers(&self, user_id: Uuid) -> Result<Vec<User>, DomainError> {
        self.db.get_followers(user_id).await
    }

    pub async fn get_following(&self, user_id: Uuid) -> Result<Vec<User>, DomainError> {
        self.db.get_following(user_id).await
    }

    pub async fn block_user(
        &self,
        blocker_id: Uuid,
        blocked_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.block_user(blocker_id, blocked_id).await
    }

    pub async fn unblock_user(
        &self,
        blocker_id: Uuid,
        blocked_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.unblock_user(blocker_id, blocked_id).await
    }

    pub async fn is_blocked_between(&self, user_a: Uuid, user_b: Uuid) -> bool {
        self.db.is_blocked_between(user_a, user_b).await
    }

    pub async fn is_blocking(&self, blocker_id: Uuid, blocked_id: Uuid) -> bool {
        self.db.is_blocking(blocker_id, blocked_id).await
    }

    pub async fn mute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError> {
        self.db.mute_user(muter_id, muted_id).await
    }

    pub async fn unmute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError> {
        self.db.unmute_user(muter_id, muted_id).await
    }

    pub async fn create_follow_request(
        &self,
        requester_id: Uuid,
        target_id: Uuid,
    ) -> Result<FollowRequest, DomainError> {
        self.db.create_follow_request(requester_id, target_id).await
    }

    pub async fn accept_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<User, DomainError> {
        self.db.accept_follow_request(target_id, requester_id).await
    }

    pub async fn reject_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.reject_follow_request(target_id, requester_id).await
    }

    pub async fn search_users(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Vec<User>, DomainError> {
        self.db.search_users(query, limit).await
    }

    pub async fn get_pending_follow_requests(
        &self,
        target_id: Uuid,
    ) -> Result<Vec<FollowRequest>, DomainError> {
        self.db.get_pending_follow_requests(target_id).await
    }

    pub async fn get_pending_follow_requests_count(&self, target_id: Uuid) -> usize {
        self.db.get_pending_follow_requests_count(target_id).await
    }

    // User Lists
    pub async fn create_user_list(
        &self,
        owner_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
        member_ids: Vec<Uuid>,
    ) -> Result<UserList, DomainError> {
        self.db
            .create_user_list(owner_id, name, description, is_private, member_ids)
            .await
    }

    pub async fn update_user_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<UserList, DomainError> {
        self.db
            .update_user_list(list_id, owner_id, name, description, is_private)
            .await
    }

    pub async fn delete_user_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.delete_user_list(list_id, owner_id).await
    }

    pub async fn add_user_to_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db.add_user_to_list(list_id, owner_id, user_id).await
    }

    pub async fn remove_user_from_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db
            .remove_user_from_list(list_id, owner_id, user_id)
            .await
    }

    pub async fn get_user_lists(&self, user_id: Uuid) -> Result<Vec<UserList>, DomainError> {
        self.db.get_user_lists(user_id).await
    }

    pub async fn get_user_list_by_id(
        &self,
        list_id: Uuid,
    ) -> Result<Option<UserList>, DomainError> {
        self.db.get_user_list_by_id(list_id).await
    }

    pub async fn get_list_members(&self, list_id: Uuid) -> Result<Vec<User>, DomainError> {
        self.db.get_list_members(list_id).await
    }

    pub async fn get_list_feed_cursor(
        &self,
        list_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<(Vec<crate::domain::models::Post>, bool), DomainError> {
        self.db.get_list_feed_cursor(list_id, first, after).await
    }
}
