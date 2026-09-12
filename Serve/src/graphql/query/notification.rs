use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::NotificationRepository;
use crate::graphql::types::{
    NotificationConnectionGql, NotificationEdgeGql, NotificationGql, PageInfoGql,
    decode_cursor, encode_cursor,
};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};

#[derive(Default)]
pub struct NotificationQuery;

#[Object]
impl NotificationQuery {
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
}
