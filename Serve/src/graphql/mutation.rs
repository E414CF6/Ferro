use crate::application::auth_service::AuthService;
use crate::application::helpers::resolve_user_id;
use crate::application::totp_service::TotpService;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::events::EventPublisher;
use crate::domain::models::{
    MediaType, NotificationType, PostAudience, PostMedia, ReportReason, ReportStatus,
    ReportTargetType, TypingEvent,
};
use crate::domain::repositories::*;
use crate::graphql::types::{
    AuthPayloadGql, BookmarkCollectionGql, CommentGql, CreatePollInput, DirectMessageGql,
    GroupConversationGql, MediaInput, PollOptionGql, PostAudienceGql, PostGql, ReportGql,
    ReportReasonGql, ReportStatusGql, ReportTargetTypeGql, StoryGql, TotpSetupGql, UserGql,
    UserListGql,
};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;

use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::Utc;
use uuid::Uuid;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new user with full profile fields and return JWT auth token
    async fn signup(
        &self,
        ctx: &Context<'_>,
        username: String,
        email: String,
        password: String,
        display_name: String,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<AuthPayloadGql> {
        let auth_service = ctx.data::<AuthService<Database>>()?;
        let (token, user) = auth_service
            .signup(
                username,
                email,
                password,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token,
            user: UserGql(user),
        })
    }

    /// Login user with username/email and password
    async fn login(
        &self,
        ctx: &Context<'_>,
        username_or_email: String,
        password: String,
    ) -> Result<AuthPayloadGql> {
        let auth_service = ctx.data::<AuthService<Database>>()?;
        let (token, user) = auth_service
            .login(&username_or_email, &password)
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token,
            user: UserGql(user),
        })
    }

    /// Update user profile details
    async fn update_user_profile(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<UserGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .update_user_profile(
                uid,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(user))
    }

    /// Update account privacy setting (public vs private account)
    async fn update_user_privacy(
        &self,
        ctx: &Context<'_>,
        is_private: bool,
        user_id: Option<ID>,
    ) -> Result<UserGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .update_user_privacy(uid, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(user))
    }

    /// Generate TOTP secret and QR/auth URI for 2FA setup
    async fn setup_2fa(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<TotpSetupGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        if user.is_2fa_enabled {
            return Err(
                DomainError::new(ErrorCode::TotpAlreadyEnabled, "2FA is already enabled").extend(),
            );
        }

        let (secret, otpauth_uri) = TotpService::generate_secret(&user.username, "Serve");
        db.set_totp_secret(uid, secret.clone())
            .await
            .map_err(|e| e.extend())?;

        Ok(TotpSetupGql {
            secret,
            otpauth_uri,
        })
    }

    /// Enable 2FA after verifying first 6-digit TOTP code
    async fn enable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP setup not initiated").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        db.enable_2fa(uid).await.map_err(|e| e.extend())?;
        Ok(true)
    }

    /// Disable 2FA by verifying 6-digit TOTP code
    async fn disable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        if !user.is_2fa_enabled {
            return Err(DomainError::new(ErrorCode::TotpNotEnabled, "2FA is not enabled").extend());
        }

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP secret missing").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        db.disable_2fa(uid).await.map_err(|e| e.extend())?;
        Ok(true)
    }

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

    /// Create an Instagram-style Story (expires in 24 hours)
    async fn create_story(
        &self,
        ctx: &Context<'_>,
        media_url: String,
        caption: Option<String>,
        author_id: Option<ID>,
    ) -> Result<StoryGql> {
        let aid = resolve_user_id(ctx, author_id)?;
        let db = ctx.data::<Database>()?;
        let story = db
            .create_story(aid, media_url, caption)
            .await
            .map_err(|e| e.extend())?;
        Ok(StoryGql(story))
    }

    /// Mark a story as viewed by current user
    async fn view_story(
        &self,
        ctx: &Context<'_>,
        story_id: ID,
        viewer_id: Option<ID>,
    ) -> Result<bool> {
        let vid = resolve_user_id(ctx, viewer_id)?;
        let sid = Uuid::parse_str(&story_id)?;
        let db = ctx.data::<Database>()?;
        db.view_story(sid, vid).await.map_err(|e| e.extend())
    }

    /// Delete a story
    async fn delete_story(
        &self,
        ctx: &Context<'_>,
        story_id: ID,
        author_id: Option<ID>,
    ) -> Result<bool> {
        let aid = resolve_user_id(ctx, author_id)?;
        let sid = Uuid::parse_str(&story_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_story(sid, aid).await.map_err(|e| e.extend())
    }

    /// Create a new post with optional media attachments, audience, and interactive poll
    async fn create_post(
        &self,
        ctx: &Context<'_>,
        content: String,
        author_id: Option<ID>,
        audience: Option<PostAudienceGql>,
        media: Option<Vec<MediaInput>>,
        poll: Option<CreatePollInput>,
    ) -> Result<PostGql> {
        let valid_content =
            crate::domain::validation::validate_post_content(&content).map_err(|e| e.extend())?;
        let aid = resolve_user_id(ctx, author_id)?;
        let db = ctx.data::<Database>()?;
        let aud_model = audience.map(PostAudience::from);
        let post = db
            .create_post(aid, valid_content, aud_model)
            .await
            .map_err(|e| e.extend())?;

        // Attach media
        if let Some(media_inputs) = media {
            let mut post_media_list = Vec::new();
            for (idx, m) in media_inputs.into_iter().enumerate() {
                post_media_list.push(PostMedia {
                    id: Uuid::new_v4(),
                    post_id: post.id,
                    media_url: m.media_url,
                    media_type: m
                        .media_type
                        .map(MediaType::from)
                        .unwrap_or(MediaType::Image),
                    alt_text: m.alt_text,
                    sort_order: m.sort_order.unwrap_or(idx as i32),
                    width: m.width,
                    height: m.height,
                    created_at: Utc::now(),
                });
            }
            let _ = db
                .add_post_media(post.id, post_media_list)
                .await
                .map_err(|e| e.extend())?;
        }

        // Attach poll
        if let Some(poll_input) = poll {
            let duration = poll_input.duration_seconds.unwrap_or(86400);
            let _ = db
                .create_poll(post.id, poll_input.question, poll_input.options, duration)
                .await
                .map_err(|e| e.extend())?;
        }

        // Automatic mention notifications
        if let Ok(broker) = ctx.data::<MessageBroker>() {
            for word in content.split_whitespace() {
                if let Some(handle) = word.strip_prefix('@') {
                    let clean: String = handle
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if let Some(target_user) = db.get_user_by_username(&clean).await {
                        if target_user.id != aid {
                            if let Ok(notif) = db
                                .create_notification(
                                    target_user.id,
                                    aid,
                                    NotificationType::Mention,
                                    Some(post.id),
                                )
                                .await
                            {
                                broker.publish_notification(notif);
                            }
                        }
                    }
                }
            }
        }

        Ok(PostGql(post))
    }

    /// Vote on a poll option
    async fn vote_poll(
        &self,
        ctx: &Context<'_>,
        poll_id: ID,
        option_id: ID,
        user_id: Option<ID>,
    ) -> Result<PollOptionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&poll_id)?;
        let oid = Uuid::parse_str(&option_id)?;
        let db = ctx.data::<Database>()?;
        let _ = db.vote_poll(pid, oid, uid).await.map_err(|e| e.extend())?;

        let options = db.get_poll_options(pid).await;
        let opt = options.into_iter().find(|o| o.id == oid).ok_or_else(|| {
            DomainError::new(ErrorCode::ErrorNotFound, "Option not found").extend()
        })?;

        Ok(PollOptionGql(opt))
    }

    /// Record a view impression on a post
    async fn record_post_view(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        viewer_id: Option<ID>,
    ) -> Result<bool> {
        let vid = resolve_user_id(ctx, viewer_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.record_post_view(pid, vid).await.map_err(|e| e.extend())
    }

    /// Quote an existing post with commentary
    async fn quote_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        content: String,
        author_id: Option<ID>,
    ) -> Result<PostGql> {
        let valid_content =
            crate::domain::validation::validate_post_content(&content).map_err(|e| e.extend())?;
        let aid = resolve_user_id(ctx, author_id)?;
        let qid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db
            .create_quote_post(aid, qid, valid_content)
            .await
            .map_err(|e| e.extend())?;

        // Trigger quote notification to original post author
        if let Ok(broker) = ctx.data::<MessageBroker>() {
            if let Some(orig_post) = db.get_post_by_id(qid).await {
                if orig_post.author_id != aid {
                    if let Ok(notif) = db
                        .create_notification(
                            orig_post.author_id,
                            aid,
                            NotificationType::Quote,
                            Some(post.id),
                        )
                        .await
                    {
                        broker.publish_notification(notif);
                    }
                }
            }
        }

        Ok(PostGql(post))
    }

    /// Repost / Retweet a post
    async fn repost_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.repost_post(uid, pid).await.map_err(|e| e.extend())?;

        // Trigger Repost notification
        if post.author_id != uid {
            if let Ok(broker) = ctx.data::<MessageBroker>() {
                if let Ok(notif) = db
                    .create_notification(
                        post.author_id,
                        uid,
                        NotificationType::Repost,
                        Some(post.id),
                    )
                    .await
                {
                    broker.publish_notification(notif);
                }
            }
        }

        Ok(PostGql(post))
    }

    /// Un-repost a post
    async fn unrepost_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.unrepost_post(uid, pid).await.map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Update an existing post by author
    async fn update_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        content: String,
        author_id: Option<ID>,
    ) -> Result<PostGql> {
        let valid_content =
            crate::domain::validation::validate_post_content(&content).map_err(|e| e.extend())?;
        let aid = resolve_user_id(ctx, author_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db
            .update_post(pid, aid, valid_content)
            .await
            .map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Delete a post by author
    async fn delete_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        author_id: Option<ID>,
    ) -> Result<bool> {
        let aid = resolve_user_id(ctx, author_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_post(pid, aid).await.map_err(|e| e.extend())
    }

    /// Add a comment or threaded reply to a post
    async fn create_comment(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        content: String,
        author_id: Option<ID>,
        parent_id: Option<ID>,
    ) -> Result<CommentGql> {
        let valid_content = crate::domain::validation::validate_comment_content(&content)
            .map_err(|e| e.extend())?;
        let aid = resolve_user_id(ctx, author_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let parent_uuid = if let Some(ref p_str) = parent_id {
            Some(Uuid::parse_str(p_str)?)
        } else {
            None
        };

        let db = ctx.data::<Database>()?;
        let comment = db
            .create_comment(pid, aid, valid_content, parent_uuid)
            .await
            .map_err(|e| e.extend())?;

        // Automatic real-time notification delivery
        if let Ok(broker) = ctx.data::<MessageBroker>() {
            if let Some(parent_uid) = parent_uuid {
                if let Some(parent_comment) = db.get_comment_by_id(parent_uid).await {
                    if parent_comment.author_id != aid {
                        if let Ok(notif) = db
                            .create_notification(
                                parent_comment.author_id,
                                aid,
                                NotificationType::ReplyComment,
                                Some(comment.id),
                            )
                            .await
                        {
                            broker.publish_notification(notif);
                        }
                    }
                }
            } else if let Some(post) = db.get_post_by_id(pid).await {
                if post.author_id != aid {
                    if let Ok(notif) = db
                        .create_notification(
                            post.author_id,
                            aid,
                            NotificationType::CommentPost,
                            Some(comment.id),
                        )
                        .await
                    {
                        broker.publish_notification(notif);
                    }
                }
            }
        }

        Ok(CommentGql(comment))
    }

    /// Like a comment
    async fn like_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        user_id: Option<ID>,
    ) -> Result<CommentGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&comment_id)?;
        let db = ctx.data::<Database>()?;
        let comment = db.like_comment(uid, cid).await.map_err(|e| e.extend())?;

        // Send notification if not liking own comment
        if comment.author_id != uid {
            if let Ok(notif) = db
                .create_notification(
                    comment.author_id,
                    uid,
                    NotificationType::LikeComment,
                    Some(comment.id),
                )
                .await
            {
                if let Ok(broker) = ctx.data::<MessageBroker>() {
                    broker.publish_notification(notif);
                }
            }
        }

        Ok(CommentGql(comment))
    }

    /// Unlike a comment
    async fn unlike_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        user_id: Option<ID>,
    ) -> Result<CommentGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&comment_id)?;
        let db = ctx.data::<Database>()?;
        let comment = db.unlike_comment(uid, cid).await.map_err(|e| e.extend())?;
        Ok(CommentGql(comment))
    }

    /// Edit an existing comment
    async fn edit_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        content: String,
        author_id: Option<ID>,
    ) -> Result<CommentGql> {
        let aid = resolve_user_id(ctx, author_id)?;
        let cid = Uuid::parse_str(&comment_id)?;
        let valid_content = crate::domain::validation::validate_comment_content(&content)
            .map_err(|e| e.extend())?;
        let db = ctx.data::<Database>()?;
        let comment = db
            .edit_comment(cid, aid, valid_content)
            .await
            .map_err(|e| e.extend())?;
        Ok(CommentGql(comment))
    }

    /// Pin a comment to the top of a post (post author only)
    async fn pin_comment(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        comment_id: ID,
        author_id: Option<ID>,
    ) -> Result<PostGql> {
        let aid = resolve_user_id(ctx, author_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let cid = Uuid::parse_str(&comment_id)?;
        let db = ctx.data::<Database>()?;
        let post = db
            .pin_comment(pid, cid, aid)
            .await
            .map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Unpin a pinned comment from a post
    async fn unpin_comment(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        author_id: Option<ID>,
    ) -> Result<PostGql> {
        let aid = resolve_user_id(ctx, author_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.unpin_comment(pid, aid).await.map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Delete a comment by author
    async fn delete_comment(
        &self,
        ctx: &Context<'_>,
        comment_id: ID,
        author_id: Option<ID>,
    ) -> Result<bool> {
        let aid = resolve_user_id(ctx, author_id)?;
        let cid = Uuid::parse_str(&comment_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_comment(cid, aid).await.map_err(|e| e.extend())
    }

    /// Like a post
    async fn like_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.like_post(uid, pid).await.map_err(|e| e.extend())?;

        // Real-time notification trigger for post author
        if post.author_id != uid {
            if let Ok(notif) = db
                .create_notification(
                    post.author_id,
                    uid,
                    NotificationType::LikePost,
                    Some(post.id),
                )
                .await
            {
                if let Ok(broker) = ctx.data::<MessageBroker>() {
                    broker.publish_notification(notif);
                }
            }
        }

        Ok(PostGql(post))
    }

    /// Unlike a post
    async fn unlike_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.unlike_post(uid, pid).await.map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Save / bookmark a post
    async fn save_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.save_post(uid, pid).await.map_err(|e| e.extend())?;
        Ok(PostGql(post))
    }

    /// Unsave / remove bookmark of a post
    async fn unsave_post(
        &self,
        ctx: &Context<'_>,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<PostGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        let post = db.unsave_post(uid, pid).await.map_err(|e| e.extend())?;
        Ok(PostGql(post))
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

    /// Create a bookmark collection
    async fn create_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        is_private: Option<bool>,
        user_id: Option<ID>,
    ) -> Result<BookmarkCollectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let coll = db
            .create_bookmark_collection(uid, name, description, is_private.unwrap_or(true))
            .await
            .map_err(|e| e.extend())?;
        Ok(BookmarkCollectionGql(coll))
    }

    /// Update a bookmark collection
    async fn update_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
        user_id: Option<ID>,
    ) -> Result<BookmarkCollectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let db = ctx.data::<Database>()?;
        let coll = db
            .update_bookmark_collection(cid, uid, name, description, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(BookmarkCollectionGql(coll))
    }

    /// Delete a bookmark collection
    async fn delete_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_bookmark_collection(cid, uid)
            .await
            .map_err(|e| e.extend())
    }

    /// Add a post to a bookmark collection
    async fn add_post_to_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.add_post_to_collection(cid, uid, pid)
            .await
            .map_err(|e| e.extend())
    }

    /// Remove a post from a bookmark collection
    async fn remove_post_from_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.remove_post_from_collection(cid, uid, pid)
            .await
            .map_err(|e| e.extend())
    }

    /// Report a post, comment, or user for moderation
    async fn report_content(
        &self,
        ctx: &Context<'_>,
        target_type: ReportTargetTypeGql,
        target_id: ID,
        reason: ReportReasonGql,
        details: Option<String>,
        reporter_id: Option<ID>,
    ) -> Result<ReportGql> {
        let rid = resolve_user_id(ctx, reporter_id)?;
        let tid = Uuid::parse_str(&target_id)?;
        let db = ctx.data::<Database>()?;
        let report = db
            .create_report(
                rid,
                ReportTargetType::from(target_type),
                tid,
                ReportReason::from(reason),
                details,
            )
            .await
            .map_err(|e| e.extend())?;
        Ok(ReportGql(report))
    }

    /// Resolve a reported item (moderation action)
    async fn resolve_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        status: ReportStatusGql,
    ) -> Result<ReportGql> {
        let rid = Uuid::parse_str(&report_id)?;
        let db = ctx.data::<Database>()?;
        let report = db
            .resolve_report(rid, ReportStatus::from(status))
            .await
            .map_err(|e| e.extend())?;
        Ok(ReportGql(report))
    }

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

    /// Mark a single notification as read
    async fn mark_notification_as_read(
        &self,
        ctx: &Context<'_>,
        notification_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let nid = Uuid::parse_str(&notification_id)?;
        let db = ctx.data::<Database>()?;
        db.mark_notification_as_read(nid, uid)
            .await
            .map_err(|e| e.extend())
    }

    /// Mark all notifications as read for current user
    async fn mark_all_notifications_as_read(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        db.mark_all_notifications_as_read(uid)
            .await
            .map_err(|e| e.extend())
    }
}
