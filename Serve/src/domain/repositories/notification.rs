use crate::domain::errors::DomainError;
use crate::domain::models::{Notification, NotificationType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create_notification(
        &self,
        recipient_id: Uuid,
        actor_id: Uuid,
        notification_type: NotificationType,
        entity_id: Option<Uuid>,
    ) -> Result<Notification, DomainError>;

    async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Notification>;

    async fn get_notifications_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Notification>, bool);

    async fn get_unread_notifications_count(&self, user_id: Uuid) -> usize;

    async fn mark_notification_as_read(
        &self,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;

    async fn mark_all_notifications_as_read(&self, user_id: Uuid) -> Result<bool, DomainError>;
}
