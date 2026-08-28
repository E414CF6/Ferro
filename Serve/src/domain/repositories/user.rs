use crate::domain::errors::DomainError;
use crate::domain::models::{FollowRequest, User};

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_users(&self) -> Vec<User>;
    async fn get_user_by_id(&self, id: Uuid) -> Option<User>;
    async fn get_user_by_username(&self, username: &str) -> Option<User>;
    async fn get_user_by_identifier(&self, identifier: &str) -> Option<User>;
    async fn register_user(
        &self,
        username: String,
        email: String,
        password_hash: String,
        display_name: String,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<User, DomainError>;
    async fn update_user_profile(
        &self,
        user_id: Uuid,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<User, DomainError>;
    async fn update_user_privacy(
        &self,
        user_id: Uuid,
        is_private: bool,
    ) -> Result<User, DomainError>;
    async fn is_following(&self, follower_id: Uuid, followee_id: Uuid) -> bool;
    async fn follow_user(&self, follower_id: Uuid, followee_id: Uuid) -> Result<User, DomainError>;
    async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<User, DomainError>;
    async fn get_followers(&self, user_id: Uuid) -> Vec<User>;
    async fn get_following(&self, user_id: Uuid) -> Vec<User>;
    async fn get_followers_count(&self, user_id: Uuid) -> usize;
    async fn get_following_count(&self, user_id: Uuid) -> usize;
    async fn search_users(&self, query: &str, limit: Option<usize>) -> Vec<User>;

    // Trust & Safety: Blocks & Mutes
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, DomainError>;
    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, DomainError>;
    async fn is_blocking(&self, blocker_id: Uuid, blocked_id: Uuid) -> bool;
    async fn is_blocked_between(&self, user_a: Uuid, user_b: Uuid) -> bool;
    async fn mute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError>;
    async fn unmute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError>;
    async fn is_muting(&self, muter_id: Uuid, muted_id: Uuid) -> bool;

    // Follow Requests
    async fn create_follow_request(
        &self,
        requester_id: Uuid,
        target_id: Uuid,
    ) -> Result<FollowRequest, DomainError>;
    async fn accept_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<User, DomainError>;
    async fn reject_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn get_pending_follow_requests(&self, target_id: Uuid) -> Vec<FollowRequest>;
    async fn has_pending_follow_request(&self, requester_id: Uuid, target_id: Uuid) -> bool;
    async fn get_pending_follow_requests_count(&self, target_id: Uuid) -> usize;

    // 2FA (Two-Factor Authentication)
    async fn set_totp_secret(&self, user_id: Uuid, secret: String) -> Result<bool, DomainError>;
    async fn enable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError>;
    async fn disable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError>;
}
