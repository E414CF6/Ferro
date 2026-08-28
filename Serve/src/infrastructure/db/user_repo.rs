use super::postgres::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{FollowRequest, FollowRequestStatus, User};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::db::entities::{FollowRequestEntity, UserEntity};

use async_trait::async_trait;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl UserRepository for Database {
    async fn get_users(&self) -> Vec<User> {
        db_fetch_all!(self, UserEntity, "SELECT * FROM users ORDER BY created_at")
            .unwrap_or_default()
            .into_iter()
            .map(User::from)
            .collect()
    }

    async fn get_user_by_id(&self, id: Uuid) -> Option<User> {
        db_fetch_optional!(self, UserEntity, "SELECT * FROM users WHERE id = $1", id)
            .ok()
            .flatten()
            .map(User::from)
    }

    async fn get_user_by_username(&self, username: &str) -> Option<User> {
        db_fetch_optional!(
            self,
            UserEntity,
            "SELECT * FROM users WHERE LOWER(username) = LOWER($1)",
            username
        )
        .ok()
        .flatten()
        .map(User::from)
    }

    async fn get_user_by_identifier(&self, identifier: &str) -> Option<User> {
        db_fetch_optional!(
            self,
            UserEntity,
            "SELECT * FROM users WHERE LOWER(username) = LOWER($1) OR LOWER(email) = LOWER($1)",
            identifier
        )
        .ok()
        .flatten()
        .map(User::from)
    }

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
    ) -> Result<User, DomainError> {
        let default_avatar = format!("https://api.dicebear.com/7.x/bottts/svg?seed={}", username);
        let user = User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            display_name,
            bio,
            avatar_url: avatar_url.or(Some(default_avatar)),
            header_image_url,
            location,
            website,
            is_private: false,
            is_2fa_enabled: false,
            totp_secret: None,
            created_at: Utc::now(),
        };

        db_execute!(
            self,
            "INSERT INTO users (id, username, email, password_hash, display_name, bio, avatar_url, header_image_url, location, website, is_private, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            user.id,
            &user.username,
            &user.email,
            &user.password_hash,
            &user.display_name,
            &user.bio,
            &user.avatar_url,
            &user.header_image_url,
            &user.location,
            &user.website,
            user.is_private,
            user.created_at
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to register user");
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() || db_err.code().is_some_and(|c| c == "23505") {
                    return DomainError::new(
                        ErrorCode::AuthUserAlreadyExists,
                        ErrorCode::AuthUserAlreadyExists.as_str(),
                    );
                }
            }
            DomainError::new(ErrorCode::ErrorInternal, ErrorCode::ErrorInternal.as_str())
        })?;

        Ok(user)
    }

    async fn update_user_profile(
        &self,
        id: Uuid,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<User, DomainError> {
        let mut user = self.get_user_by_id(id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, ErrorCode::UserNotFound.as_str())
        })?;

        if let Some(dn) = display_name {
            user.display_name = dn;
        }
        if bio.is_some() {
            user.bio = bio;
        }
        if avatar_url.is_some() {
            user.avatar_url = avatar_url;
        }
        if header_image_url.is_some() {
            user.header_image_url = header_image_url;
        }
        if location.is_some() {
            user.location = location;
        }
        if website.is_some() {
            user.website = website;
        }

        db_execute!(
            self,
            "UPDATE users SET display_name = $1, bio = $2, avatar_url = $3, header_image_url = $4, location = $5, website = $6 WHERE id = $7",
            &user.display_name,
            &user.bio,
            &user.avatar_url,
            &user.header_image_url,
            &user.location,
            &user.website,
            id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update profile");
            DomainError::new(ErrorCode::UserProfileUpdateFailed, ErrorCode::UserProfileUpdateFailed.as_str())
        })?;

        Ok(user)
    }

    async fn update_user_privacy(
        &self,
        user_id: Uuid,
        is_private: bool,
    ) -> Result<User, DomainError> {
        let updated = db_fetch_one!(
            self,
            UserEntity,
            "UPDATE users SET is_private = $1 WHERE id = $2 RETURNING *",
            is_private,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to update privacy");
            DomainError::new(
                ErrorCode::UserProfileUpdateFailed,
                ErrorCode::UserProfileUpdateFailed.as_str(),
            )
        })?;

        Ok(User::from(updated))
    }

    async fn is_following(&self, follower_id: Uuid, followee_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM follows WHERE follower_id = $1 AND followee_id = $2)",
            follower_id,
            followee_id
        )
        .unwrap_or(false)
    }

    async fn follow_user(&self, follower_id: Uuid, followee_id: Uuid) -> Result<User, DomainError> {
        let followee = self.get_user_by_id(followee_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, ErrorCode::UserNotFound.as_str())
        })?;

        if self.is_blocked_between(follower_id, followee_id).await {
            return Err(DomainError::new(
                ErrorCode::UserBlocked,
                "Cannot follow blocked user",
            ));
        }

        let _ = db_execute!(
            self,
            "INSERT INTO follows (follower_id, followee_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            follower_id,
            followee_id,
            Utc::now()
        );

        Ok(followee)
    }

    async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<User, DomainError> {
        let followee = self.get_user_by_id(followee_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, ErrorCode::UserNotFound.as_str())
        })?;

        let _ = db_execute!(
            self,
            "DELETE FROM follows WHERE follower_id = $1 AND followee_id = $2",
            follower_id,
            followee_id
        );

        Ok(followee)
    }

    async fn get_followers(&self, user_id: Uuid) -> Vec<User> {
        db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u JOIN follows f ON u.id = f.follower_id WHERE f.followee_id = $1",
            user_id
        )
        .unwrap_or_default()
        .into_iter()
        .map(User::from)
        .collect()
    }

    async fn get_following(&self, user_id: Uuid) -> Vec<User> {
        db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u JOIN follows f ON u.id = f.followee_id WHERE f.follower_id = $1",
            user_id
        )
        .unwrap_or_default()
        .into_iter()
        .map(User::from)
        .collect()
    }

    async fn get_followers_count(&self, user_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM follows WHERE followee_id = $1",
            user_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_following_count(&self, user_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM follows WHERE follower_id = $1",
            user_id
        )
        .unwrap_or(0) as usize
    }

    async fn search_users(&self, query: &str, limit: Option<usize>) -> Vec<User> {
        let lim = limit.unwrap_or(20) as i64;
        let pattern = format!("%{}%", query.to_lowercase());
        db_fetch_all!(
            self,
            UserEntity,
            "SELECT * FROM users WHERE LOWER(username) LIKE $1 OR LOWER(display_name) LIKE $1 ORDER BY created_at DESC LIMIT $2",
            pattern,
            lim
        )
        .unwrap_or_default()
        .into_iter()
        .map(User::from)
        .collect()
    }

    // Trust & Safety: Blocks
    async fn block_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, DomainError> {
        if blocker_id == blocked_id {
            return Err(DomainError::new(
                ErrorCode::CannotBlockSelf,
                "Cannot block yourself",
            ));
        }

        // Remove follow relationships in both directions upon blocking
        let _ = db_execute!(
            self,
            "DELETE FROM follows WHERE (follower_id = $1 AND followee_id = $2) OR (follower_id = $2 AND followee_id = $1)",
            blocker_id,
            blocked_id
        );

        db_execute!(
            self,
            "INSERT INTO blocks (blocker_id, blocked_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            blocker_id,
            blocked_id,
            Utc::now()
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to block user");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to block user")
        })?;

        Ok(true)
    }

    async fn unblock_user(&self, blocker_id: Uuid, blocked_id: Uuid) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "DELETE FROM blocks WHERE blocker_id = $1 AND blocked_id = $2",
            blocker_id,
            blocked_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to unblock user");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to unblock user")
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::BlockNotFound,
                "User is not blocked",
            ))
        }
    }

    async fn is_blocking(&self, blocker_id: Uuid, blocked_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM blocks WHERE blocker_id = $1 AND blocked_id = $2)",
            blocker_id,
            blocked_id
        )
        .unwrap_or(false)
    }

    async fn is_blocked_between(&self, user_a: Uuid, user_b: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM blocks WHERE (blocker_id = $1 AND blocked_id = $2) OR (blocker_id = $2 AND blocked_id = $1))",
            user_a,
            user_b
        )
        .unwrap_or(false)
    }

    // Trust & Safety: Mutes
    async fn mute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError> {
        if muter_id == muted_id {
            return Err(DomainError::new(
                ErrorCode::CannotMuteSelf,
                "Cannot mute yourself",
            ));
        }

        db_execute!(
            self,
            "INSERT INTO mutes (muter_id, muted_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            muter_id,
            muted_id,
            Utc::now()
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to mute user");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to mute user")
        })?;

        Ok(true)
    }

    async fn unmute_user(&self, muter_id: Uuid, muted_id: Uuid) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "DELETE FROM mutes WHERE muter_id = $1 AND muted_id = $2",
            muter_id,
            muted_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to unmute user");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to unmute user")
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::MuteNotFound,
                "User is not muted",
            ))
        }
    }

    async fn is_muting(&self, muter_id: Uuid, muted_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM mutes WHERE muter_id = $1 AND muted_id = $2)",
            muter_id,
            muted_id
        )
        .unwrap_or(false)
    }

    // Follow Requests for Private Accounts
    async fn create_follow_request(
        &self,
        requester_id: Uuid,
        target_id: Uuid,
    ) -> Result<FollowRequest, DomainError> {
        if requester_id == target_id {
            return Err(DomainError::new(
                ErrorCode::FollowRequestCannotTargetSelf,
                "Cannot send follow request to yourself",
            ));
        }

        let request = FollowRequest {
            id: Uuid::new_v4(),
            requester_id,
            target_id,
            status: FollowRequestStatus::Pending,
            created_at: Utc::now(),
        };

        db_execute!(
            self,
            "INSERT INTO follow_requests (id, requester_id, target_id, status, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (requester_id, target_id) DO UPDATE SET status = 'PENDING', created_at = EXCLUDED.created_at",
            request.id,
            request.requester_id,
            request.target_id,
            request.status.as_str(),
            request.created_at
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create follow request");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to create follow request")
        })?;

        Ok(request)
    }

    async fn accept_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<User, DomainError> {
        let requester = self
            .get_user_by_id(requester_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "Requester not found"))?;

        let rows = db_execute!(
            self,
            "DELETE FROM follow_requests WHERE target_id = $1 AND requester_id = $2",
            target_id,
            requester_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete follow request");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to process follow request")
        })?;

        if rows == 0 {
            return Err(DomainError::new(
                ErrorCode::FollowRequestNotFound,
                "Follow request not found",
            ));
        }

        // Insert follow record: requester follows target
        let _ = db_execute!(
            self,
            "INSERT INTO follows (follower_id, followee_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            requester_id,
            target_id,
            Utc::now()
        );

        Ok(requester)
    }

    async fn reject_follow_request(
        &self,
        target_id: Uuid,
        requester_id: Uuid,
    ) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "DELETE FROM follow_requests WHERE target_id = $1 AND requester_id = $2",
            target_id,
            requester_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete follow request");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to reject follow request")
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::FollowRequestNotFound,
                "Follow request not found",
            ))
        }
    }

    async fn get_pending_follow_requests(&self, target_id: Uuid) -> Vec<FollowRequest> {
        db_fetch_all!(
            self,
            FollowRequestEntity,
            "SELECT * FROM follow_requests WHERE target_id = $1 AND status = 'PENDING' ORDER BY created_at DESC",
            target_id
        )
        .unwrap_or_default()
        .into_iter()
        .map(FollowRequest::from)
        .collect()
    }

    async fn has_pending_follow_request(&self, requester_id: Uuid, target_id: Uuid) -> bool {
        db_scalar!(
            self,
            bool,
            "SELECT EXISTS(SELECT 1 FROM follow_requests WHERE requester_id = $1 AND target_id = $2 AND status = 'PENDING')",
            requester_id,
            target_id
        )
        .unwrap_or(false)
    }

    async fn get_pending_follow_requests_count(&self, target_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM follow_requests WHERE target_id = $1 AND status = 'PENDING'",
            target_id
        )
        .unwrap_or(0) as usize
    }

    async fn set_totp_secret(&self, user_id: Uuid, secret: String) -> Result<bool, DomainError> {
        db_execute!(
            self,
            "UPDATE users SET totp_secret = $1 WHERE id = $2",
            secret,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to set TOTP secret");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to set TOTP secret")
        })?;
        Ok(true)
    }

    async fn enable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError> {
        db_execute!(
            self,
            "UPDATE users SET is_2fa_enabled = true WHERE id = $1",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to enable 2FA");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to enable 2FA")
        })?;
        Ok(true)
    }

    async fn disable_2fa(&self, user_id: Uuid) -> Result<bool, DomainError> {
        db_execute!(
            self,
            "UPDATE users SET is_2fa_enabled = false, totp_secret = NULL WHERE id = $1",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to disable 2FA");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to disable 2FA")
        })?;
        Ok(true)
    }
}
