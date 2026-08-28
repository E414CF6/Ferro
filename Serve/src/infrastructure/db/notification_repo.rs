use super::postgres::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Notification, NotificationType};
use crate::domain::repositories::NotificationRepository;
use crate::infrastructure::db::entities::NotificationEntity;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl NotificationRepository for Database {
    async fn create_notification(
        &self,
        recipient_id: Uuid,
        actor_id: Uuid,
        notification_type: NotificationType,
        entity_id: Option<Uuid>,
    ) -> Result<Notification, DomainError> {
        let notif = Notification {
            id: Uuid::new_v4(),
            recipient_id,
            actor_id,
            notification_type,
            entity_id,
            is_read: false,
            created_at: Utc::now(),
        };

        db_execute!(
            self,
            "INSERT INTO notifications (id, recipient_id, actor_id, notification_type, entity_id, is_read, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            notif.id,
            notif.recipient_id,
            notif.actor_id,
            notif.notification_type.as_str(),
            notif.entity_id,
            notif.is_read,
            notif.created_at
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create notification");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to create notification")
        })?;

        Ok(notif)
    }

    async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Notification> {
        let lim = limit.unwrap_or(50) as i64;
        let off = offset.unwrap_or(0) as i64;
        db_fetch_all!(
            self,
            NotificationEntity,
            "SELECT * FROM notifications WHERE recipient_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3",
            user_id,
            lim,
            off
        )
        .unwrap_or_default()
        .into_iter()
        .map(Notification::from)
        .collect()
    }

    async fn get_notifications_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Notification>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                NotificationEntity,
                "SELECT * FROM notifications
                     WHERE recipient_id = $1
                       AND ((created_at < $2) OR (created_at = $2 AND id < $3))
                     ORDER BY created_at DESC, id DESC
                     LIMIT $4",
                user_id,
                after_time,
                after_id,
                fetch_limit
            )
            .unwrap_or_default(),
            None => db_fetch_all!(
                self,
                NotificationEntity,
                "SELECT * FROM notifications
                     WHERE recipient_id = $1
                     ORDER BY created_at DESC, id DESC
                     LIMIT $2",
                user_id,
                fetch_limit
            )
            .unwrap_or_default(),
        };

        let has_next_page = entities.len() > first;
        let items: Vec<Notification> = entities
            .into_iter()
            .take(first)
            .map(Notification::from)
            .collect();

        (items, has_next_page)
    }

    async fn get_unread_notifications_count(&self, user_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM notifications WHERE recipient_id = $1 AND is_read = false",
            user_id
        )
        .unwrap_or(0) as usize
    }

    async fn mark_notification_as_read(
        &self,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "UPDATE notifications SET is_read = true WHERE id = $1 AND recipient_id = $2",
            notification_id,
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to mark notification as read");
            DomainError::new(
                ErrorCode::NotificationMarkReadFailed,
                ErrorCode::NotificationMarkReadFailed.as_str(),
            )
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::NotificationNotFound,
                ErrorCode::NotificationNotFound.as_str(),
            ))
        }
    }

    async fn mark_all_notifications_as_read(&self, user_id: Uuid) -> Result<bool, DomainError> {
        db_execute!(
            self,
            "UPDATE notifications SET is_read = true WHERE recipient_id = $1 AND is_read = false",
            user_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to mark all notifications as read");
            DomainError::new(
                ErrorCode::NotificationMarkReadFailed,
                ErrorCode::NotificationMarkReadFailed.as_str(),
            )
        })?;

        Ok(true)
    }
}
