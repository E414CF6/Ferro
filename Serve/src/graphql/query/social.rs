use crate::application::helpers::{require_auth, resolve_user_id};
use crate::domain::repositories::{PostRepository, UserRepository};
use crate::graphql::types::{
    FollowRequestGql, PageInfoGql, PostConnectionGql, PostEdgeGql, PostGql, UserListGql,
    decode_cursor, encode_cursor,
};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct SocialQuery;

#[Object]
impl SocialQuery {
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
}
