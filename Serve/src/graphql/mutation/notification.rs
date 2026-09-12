use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::NotificationRepository;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct NotificationMutation;

#[Object]
impl NotificationMutation {
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
