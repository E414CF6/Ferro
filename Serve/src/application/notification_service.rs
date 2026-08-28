#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{Notification, NotificationType};
use crate::domain::repositories::NotificationRepository;
use crate::infrastructure::db::postgres::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating User Notifications and read states
#[derive(Clone)]
pub struct NotificationService {
    db: Arc<Database>,
}

impl NotificationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_notification(
        &self,
        recipient_id: Uuid,
        actor_id: Uuid,
        notification_type: NotificationType,
        entity_id: Option<Uuid>,
    ) -> Result<Notification, DomainError> {
        self.db
            .create_notification(recipient_id, actor_id, notification_type, entity_id)
            .await
    }

    pub async fn get_notifications_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Notification>, bool) {
        self.db
            .get_notifications_cursor(user_id, first, after)
            .await
    }

    pub async fn get_unread_notifications_count(&self, user_id: Uuid) -> usize {
        self.db.get_unread_notifications_count(user_id).await
    }

    pub async fn mark_notification_as_read(
        &self,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        self.db
            .mark_notification_as_read(notification_id, user_id)
            .await
    }

    pub async fn mark_all_notifications_as_read(&self, user_id: Uuid) -> Result<bool, DomainError> {
        self.db.mark_all_notifications_as_read(user_id).await
    }
}
