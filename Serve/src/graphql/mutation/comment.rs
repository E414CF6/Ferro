use crate::application::helpers::resolve_user_id;
use crate::domain::models::NotificationType;
use crate::domain::repositories::{CommentRepository, NotificationRepository, PostRepository};
use crate::graphql::types::{CommentGql, PostGql};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct CommentMutation;

#[Object]
impl CommentMutation {
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
}
