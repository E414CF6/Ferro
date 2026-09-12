use crate::application::helpers::resolve_user_id;
use crate::domain::models::{MediaType, NotificationType, PostAudience, PostMedia};
use crate::domain::repositories::{
    BookmarkRepository, NotificationRepository, PollRepository, PostRepository, UserRepository,
};
use crate::graphql::types::{CreatePollInput, MediaInput, PostAudienceGql, PostGql};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::Utc;
use uuid::Uuid;

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
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
}
