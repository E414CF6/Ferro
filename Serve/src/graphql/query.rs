use crate::application::helpers::{require_auth, resolve_user_id};
use crate::domain::models::ReportStatus;
use crate::domain::repositories::*;
use crate::graphql::types::{
    BookmarkCollectionGql, CommentGql, ConversationGql, DirectMessageConnectionGql,
    DirectMessageEdgeGql, DirectMessageGql, FollowRequestGql, GroupConversationGql,
    HashtagTrendGql, NotificationConnectionGql, NotificationEdgeGql, NotificationGql, PageInfoGql,
    PollGql, PostAnalyticsGql, PostConnectionGql, PostEdgeGql, PostGql, ReportGql, ReportStatusGql,
    StoryGql, UserGql, UserListGql, decode_cursor, encode_cursor,
};
use crate::infrastructure::db::postgres::Database;

use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get current authenticated user (via Authorization JWT header) or specified user ID
    async fn me(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Option<UserGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_id(uid).await.map(UserGql))
    }

    /// Get user profile by ID
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<UserGql>> {
        let db = ctx.data::<Database>()?;
        let uid = Uuid::parse_str(&id)?;
        Ok(db.get_user_by_id(uid).await.map(UserGql))
    }

    /// Get user profile by username handle (e.g. "ferro_dev")
    async fn profile(&self, ctx: &Context<'_>, username: String) -> Result<Option<UserGql>> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_username(&username).await.map(UserGql))
    }

    /// List all registered users
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.get_users().await;
        Ok(users.into_iter().map(UserGql).collect())
    }

    /// Search users by username or display name
    async fn search_users(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.search_users(&query, limit).await;
        Ok(users.into_iter().map(UserGql).collect())
    }

    /// Get single story by ID
    async fn story(&self, ctx: &Context<'_>, id: ID) -> Result<Option<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let sid = Uuid::parse_str(&id)?;
        Ok(db.get_story_by_id(sid).await.map(StoryGql))
    }

    /// Get all active stories for a user
    async fn stories_for_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let uid = Uuid::parse_str(&user_id)?;
        let stories = db.get_active_stories_for_user(uid).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Stories tray/feed: returns all active stories from followed users and self
    async fn stories_feed(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Vec<StoryGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let stories = db.get_stories_feed_for_user(uid).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Get single post by ID
    async fn post(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PostGql>> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&id)?;
        Ok(db.get_post_by_id(pid).await.map(PostGql))
    }

    /// Get post analytics and engagement metrics
    async fn post_analytics(&self, ctx: &Context<'_>, post_id: ID) -> Result<PostAnalyticsGql> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&post_id)?;
        let an = db.get_post_analytics(pid).await.map_err(|e| e.extend())?;
        Ok(PostAnalyticsGql(an))
    }

    /// Get poll details by ID
    async fn poll(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PollGql>> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&id)?;
        Ok(db.get_poll_by_id(pid).await.map(PollGql))
    }

    /// List all posts with offset pagination
    async fn posts(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_posts(limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// List all posts with cursor-based pagination
    async fn posts_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_posts_cursor(page_size, cursor_pair).await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Timeline feed with offset pagination
    async fn feed(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let posts = db.get_feed(uid, limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// Timeline feed with cursor-based pagination
    async fn feed_connection(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_feed_cursor(uid, page_size, cursor_pair).await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Get posts saved / bookmarked by current user with cursor pagination
    async fn saved_posts_connection(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_saved_posts_cursor(uid, page_size, cursor_pair).await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Search posts containing full-text keyword with cursor pagination
    async fn search_posts_connection(
        &self,
        ctx: &Context<'_>,
        query: String,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.search_posts_cursor(&query, page_size, cursor_pair).await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Get posts associated with a hashtag with cursor pagination
    async fn posts_by_hashtag(
        &self,
        ctx: &Context<'_>,
        hashtag: String,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db
            .get_posts_by_hashtag_cursor(&hashtag, page_size, cursor_pair)
            .await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Get trending hashtags
    async fn trending_hashtags(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
    ) -> Result<Vec<HashtagTrendGql>> {
        let db = ctx.data::<Database>()?;
        let list = db.get_trending_hashtags(limit).await;
        Ok(list
            .into_iter()
            .map(|(t, c)| HashtagTrendGql { tag: t, count: c })
            .collect())
    }

    /// Get single comment by ID
    async fn comment(&self, ctx: &Context<'_>, id: ID) -> Result<Option<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let cid = Uuid::parse_str(&id)?;
        Ok(db.get_comment_by_id(cid).await.map(CommentGql))
    }

    /// Get notifications for current user with cursor pagination
    async fn notifications_connection(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<NotificationConnectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (notifs, has_next_page) = db
            .get_notifications_cursor(uid, page_size, cursor_pair)
            .await;
        let unread_count = db.get_unread_notifications_count(uid).await;
        let start_cursor = notifs.first().map(|n| encode_cursor(n.created_at, n.id));
        let end_cursor = notifs.last().map(|n| encode_cursor(n.created_at, n.id));

        let edges = notifs
            .into_iter()
            .map(|n| {
                let cursor = encode_cursor(n.created_at, n.id);
                NotificationEdgeGql {
                    cursor,
                    node: NotificationGql(n),
                }
            })
            .collect();

        Ok(NotificationConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            unread_count,
        })
    }

    /// Get unread notifications count for current user
    async fn unread_notifications_count(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
    ) -> Result<usize> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        Ok(db.get_unread_notifications_count(uid).await)
    }

    /// Get recent notifications list
    async fn notifications(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<NotificationGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let notifs = db.get_notifications(uid, limit, offset).await;
        Ok(notifs.into_iter().map(NotificationGql).collect())
    }

    /// Get conversation summaries for current user
    async fn conversations(&self, ctx: &Context<'_>) -> Result<Vec<ConversationGql>> {
        let uid = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        let convos = db.get_conversations(uid).await;
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
        let db = ctx.data::<Database>()?;
        let msgs = db.get_direct_messages(uid, other_id, limit, offset).await;
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
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (msgs, has_next_page) = db
            .get_direct_messages_cursor(uid, other_id, page_size, cursor_pair)
            .await;
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
        let db = ctx.data::<Database>()?;
        let convos = db.get_user_group_conversations(uid).await;
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
        let db = ctx.data::<Database>()?;
        Ok(db
            .get_conversation_by_id(cid)
            .await
            .map(GroupConversationGql))
    }

    /// Get total unread direct message count for current user
    async fn unread_dm_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let uid = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        Ok(db.get_unread_dm_count(uid).await)
    }

    /// Get pending follow requests for private account
    async fn pending_follow_requests(&self, ctx: &Context<'_>) -> Result<Vec<FollowRequestGql>> {
        let uid = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        let reqs = db.get_pending_follow_requests(uid).await;
        Ok(reqs.into_iter().map(FollowRequestGql).collect())
    }

    /// Get pending follow requests count for private account
    async fn pending_follow_requests_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let uid = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        Ok(db.get_pending_follow_requests_count(uid).await)
    }

    /// Get user lists
    async fn user_lists(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Vec<UserListGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let lists = db.get_user_lists(uid).await;
        Ok(lists.into_iter().map(UserListGql).collect())
    }

    /// Get user list by ID
    async fn user_list(&self, ctx: &Context<'_>, id: ID) -> Result<Option<UserListGql>> {
        let db = ctx.data::<Database>()?;
        let lid = Uuid::parse_str(&id)?;
        Ok(db.get_user_list_by_id(lid).await.map(UserListGql))
    }

    /// Get dedicated feed for a user list with cursor pagination
    async fn list_feed_connection(
        &self,
        ctx: &Context<'_>,
        list_id: ID,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let lid = Uuid::parse_str(&list_id)?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_list_feed_cursor(lid, page_size, cursor_pair).await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    /// Get bookmark collections for a user
    async fn bookmark_collections(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
    ) -> Result<Vec<BookmarkCollectionGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let colls = db.get_user_collections(uid).await;
        Ok(colls.into_iter().map(BookmarkCollectionGql).collect())
    }

    /// Get bookmark collection by ID
    async fn bookmark_collection(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<BookmarkCollectionGql>> {
        let db = ctx.data::<Database>()?;
        let cid = Uuid::parse_str(&id)?;
        Ok(db
            .get_collection_by_id(cid)
            .await
            .map(BookmarkCollectionGql))
    }

    /// Get content reports list (moderation query)
    async fn reports(
        &self,
        ctx: &Context<'_>,
        status: Option<ReportStatusGql>,
        limit: Option<usize>,
    ) -> Result<Vec<ReportGql>> {
        let _ = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        let st_model = status.map(ReportStatus::from);
        let list = db.get_reports(st_model, limit).await;
        Ok(list.into_iter().map(ReportGql).collect())
    }
}
