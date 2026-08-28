use super::postgres::{Database, DatabaseBackend};
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Conversation, ConversationSummary, DirectMessage, User};
use crate::domain::repositories::DmRepository;
use crate::infrastructure::db::entities::{ConversationEntity, DirectMessageEntity, UserEntity};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl DmRepository for Database {
    async fn send_direct_message(
        &self,
        sender_id: Uuid,
        recipient_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError> {
        let msg_id = Uuid::new_v4();
        let now = Utc::now();
        let msg = db_fetch_one!(
            self,
            DirectMessageEntity,
            "INSERT INTO direct_messages (id, sender_id, recipient_id, content, is_read, is_edited, created_at)
             VALUES ($1, $2, $3, $4, false, false, $5) RETURNING *",
            msg_id,
            sender_id,
            recipient_id,
            content,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to send direct message");
            DomainError::new(ErrorCode::DmSendFailed, ErrorCode::DmSendFailed.as_str())
        })?;
        Ok(DirectMessage::from(msg))
    }

    async fn get_direct_messages(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<DirectMessage> {
        let limit = limit.unwrap_or(50) as i64;
        let offset = offset.unwrap_or(0) as i64;
        db_fetch_all!(
            self,
            DirectMessageEntity,
            "SELECT * FROM direct_messages
             WHERE (sender_id = $1 AND recipient_id = $2) OR (sender_id = $2 AND recipient_id = $1)
             ORDER BY created_at ASC, id ASC LIMIT $3 OFFSET $4",
            user1_id,
            user2_id,
            limit,
            offset
        )
        .unwrap_or_default()
        .into_iter()
        .map(DirectMessage::from)
        .collect()
    }

    async fn get_direct_messages_cursor(
        &self,
        user1_id: Uuid,
        user2_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities = match after {
            Some((after_time, after_id)) => {
                db_fetch_all!(
                    self,
                    DirectMessageEntity,
                    "SELECT * FROM direct_messages
                     WHERE ((sender_id = $1 AND recipient_id = $2) OR (sender_id = $2 AND recipient_id = $1))
                       AND ((created_at > $3) OR (created_at = $3 AND id > $4))\
                     ORDER BY created_at ASC, id ASC
                     LIMIT $5",
                    user1_id,
                    user2_id,
                    after_time,
                    after_id,
                    fetch_limit
                )
                .unwrap_or_default()
            }
            None => {
                db_fetch_all!(
                    self,
                    DirectMessageEntity,
                    "SELECT * FROM direct_messages
                     WHERE (sender_id = $1 AND recipient_id = $2) OR (sender_id = $2 AND recipient_id = $1)
                     ORDER BY created_at ASC, id ASC
                     LIMIT $3",
                    user1_id,
                    user2_id,
                    fetch_limit
                )
                .unwrap_or_default()
            }
        };

        let has_next_page = entities.len() > first;
        let items: Vec<DirectMessage> = entities
            .into_iter()
            .take(first)
            .map(DirectMessage::from)
            .collect();

        (items, has_next_page)
    }

    async fn get_conversations(&self, user_id: Uuid) -> Vec<ConversationSummary> {
        let sql = r#"
            WITH user_conversations AS (
                SELECT partner_id, msg_id, sender_id, recipient_id, conversation_id, content, is_read, is_edited, created_at
                FROM (
                    SELECT
                        CASE WHEN sender_id = $1 THEN recipient_id ELSE sender_id END AS partner_id,
                        id AS msg_id, sender_id, recipient_id, conversation_id, content, is_read, is_edited, created_at,
                        ROW_NUMBER() OVER (PARTITION BY (CASE WHEN sender_id = $1 THEN recipient_id ELSE sender_id END) ORDER BY created_at DESC) as rn
                    FROM direct_messages
                    WHERE sender_id = $1 OR recipient_id = $1
                ) raw_msgs
                WHERE rn = 1
            ),
            unread_counts AS (
                SELECT sender_id AS partner_id, COUNT(*) AS unread_count
                FROM direct_messages
                WHERE recipient_id = $1 AND is_read = false
                GROUP BY sender_id
            )
            SELECT
                u.id as u_id, u.username as u_username, u.email as u_email, u.password_hash as u_password_hash,
                u.display_name as u_display_name, u.bio as u_bio, u.avatar_url as u_avatar_url,
                u.header_image_url as u_header_image_url, u.location as u_location, u.website as u_website,
                u.is_private as u_is_private, u.created_at as u_created_at,
                c.msg_id, c.sender_id, c.recipient_id, c.conversation_id, c.content, c.is_read, c.is_edited, c.created_at as msg_created_at,
                COALESCE(uc.unread_count, 0) as unread_count
            FROM user_conversations c
            JOIN users u ON u.id = c.partner_id
            LEFT JOIN unread_counts uc ON uc.partner_id = c.partner_id
            ORDER BY c.created_at DESC
        "#;

        match self.backend() {
            DatabaseBackend::Postgres(pool) => {
                let rows = sqlx::query(sql)
                    .bind(user_id)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                rows.into_iter()
                    .map(|row| ConversationSummary {
                        other_user: User {
                            id: row.get("u_id"),
                            username: row.get("u_username"),
                            email: row.get("u_email"),
                            password_hash: row.get("u_password_hash"),
                            display_name: row.get("u_display_name"),
                            bio: row.get("u_bio"),
                            avatar_url: row.get("u_avatar_url"),
                            header_image_url: row.get("u_header_image_url"),
                            location: row.get("u_location"),
                            website: row.get("u_website"),
                            is_private: row.get("u_is_private"),
                            is_2fa_enabled: false,
                            totp_secret: None,
                            created_at: row.get("u_created_at"),
                        },
                        last_message: DirectMessage {
                            id: row.get("msg_id"),
                            sender_id: row.get("sender_id"),
                            recipient_id: row.get("recipient_id"),
                            conversation_id: row.get("conversation_id"),
                            content: row.get("content"),
                            is_read: row.get("is_read"),
                            is_edited: row.get("is_edited"),
                            created_at: row.get("msg_created_at"),
                        },
                        unread_count: row.get("unread_count"),
                    })
                    .collect()
            }
            DatabaseBackend::Sqlite(pool) => {
                let rows = sqlx::query(sql)
                    .bind(user_id)
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                rows.into_iter()
                    .map(|row| ConversationSummary {
                        other_user: User {
                            id: row.get("u_id"),
                            username: row.get("u_username"),
                            email: row.get("u_email"),
                            password_hash: row.get("u_password_hash"),
                            display_name: row.get("u_display_name"),
                            bio: row.get("u_bio"),
                            avatar_url: row.get("u_avatar_url"),
                            header_image_url: row.get("u_header_image_url"),
                            location: row.get("u_location"),
                            website: row.get("u_website"),
                            is_private: row.get("u_is_private"),
                            is_2fa_enabled: false,
                            totp_secret: None,
                            created_at: row.get("u_created_at"),
                        },
                        last_message: DirectMessage {
                            id: row.get("msg_id"),
                            sender_id: row.get("sender_id"),
                            recipient_id: row.get("recipient_id"),
                            conversation_id: row.get("conversation_id"),
                            content: row.get("content"),
                            is_read: row.get("is_read"),
                            is_edited: row.get("is_edited"),
                            created_at: row.get("msg_created_at"),
                        },
                        unread_count: row.get("unread_count"),
                    })
                    .collect()
            }
        }
    }

    async fn mark_direct_messages_as_read(
        &self,
        reader_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError> {
        db_execute!(
            self,
            "UPDATE direct_messages SET is_read = true WHERE recipient_id = $1 AND sender_id = $2 AND is_read = false",
            reader_id,
            sender_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to mark messages as read");
            DomainError::new(ErrorCode::DmMarkReadFailed, ErrorCode::DmMarkReadFailed.as_str())
        })?;
        Ok(true)
    }

    async fn get_unread_dm_count(&self, user_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM direct_messages WHERE recipient_id = $1 AND is_read = false",
            user_id
        )
        .unwrap_or(0) as usize
    }

    async fn create_group_conversation(
        &self,
        creator_id: Uuid,
        title: Option<String>,
        participant_ids: Vec<Uuid>,
    ) -> Result<Conversation, DomainError> {
        let convo = Conversation {
            id: Uuid::new_v4(),
            is_group: true,
            title,
            created_by: Some(creator_id),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db_execute!(
            self,
            "INSERT INTO conversations (id, is_group, title, created_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            convo.id,
            convo.is_group,
            &convo.title,
            convo.created_by,
            convo.created_at,
            convo.updated_at
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create conversation");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to create conversation")
        })?;

        // Add creator and participants
        let mut all_members = participant_ids;
        if !all_members.contains(&creator_id) {
            all_members.push(creator_id);
        }

        let now = Utc::now();
        for member_id in all_members {
            let _ = db_execute!(
                self,
                "INSERT INTO conversation_participants (conversation_id, user_id, joined_at, last_read_at)
                 VALUES ($1, $2, $3, $3) ON CONFLICT DO NOTHING",
                convo.id,
                member_id,
                now
            );
        }

        Ok(convo)
    }

    async fn get_conversation_by_id(&self, conversation_id: Uuid) -> Option<Conversation> {
        db_fetch_optional!(
            self,
            ConversationEntity,
            "SELECT * FROM conversations WHERE id = $1",
            conversation_id
        )
        .ok()
        .flatten()
        .map(Conversation::from)
    }

    async fn get_conversation_participants(&self, conversation_id: Uuid) -> Vec<User> {
        db_fetch_all!(
            self,
            UserEntity,
            "SELECT u.* FROM users u
             JOIN conversation_participants cp ON u.id = cp.user_id
             WHERE cp.conversation_id = $1
             ORDER BY cp.joined_at ASC",
            conversation_id
        )
        .unwrap_or_default()
        .into_iter()
        .map(User::from)
        .collect()
    }

    async fn send_conversation_message(
        &self,
        sender_id: Uuid,
        conversation_id: Uuid,
        content: String,
    ) -> Result<DirectMessage, DomainError> {
        let msg_id = Uuid::new_v4();
        let now = Utc::now();
        let msg = db_fetch_one!(
            self,
            DirectMessageEntity,
            "INSERT INTO direct_messages (id, sender_id, recipient_id, conversation_id, content, is_read, is_edited, created_at)
             VALUES ($1, $2, $2, $3, $4, false, false, $5) RETURNING *",
            msg_id,
            sender_id,
            conversation_id,
            content,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to send conversation message");
            DomainError::new(ErrorCode::DmSendFailed, ErrorCode::DmSendFailed.as_str())
        })?;

        // Touch conversation updated_at
        let _ = db_execute!(
            self,
            "UPDATE conversations SET updated_at = $1 WHERE id = $2",
            now,
            conversation_id
        );

        Ok(DirectMessage::from(msg))
    }

    async fn get_conversation_messages(
        &self,
        conversation_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<DirectMessage> {
        let limit = limit.unwrap_or(50) as i64;
        let offset = offset.unwrap_or(0) as i64;
        db_fetch_all!(
            self,
            DirectMessageEntity,
            "SELECT * FROM direct_messages WHERE conversation_id = $1 ORDER BY created_at ASC, id ASC LIMIT $2 OFFSET $3",
            conversation_id,
            limit,
            offset
        )
        .unwrap_or_default()
        .into_iter()
        .map(DirectMessage::from)
        .collect()
    }

    async fn get_conversation_messages_cursor(
        &self,
        conversation_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<DirectMessage>, bool) {
        let fetch_limit = (first + 1) as i64;
        let entities = match after {
            Some((after_time, after_id)) => db_fetch_all!(
                self,
                DirectMessageEntity,
                "SELECT * FROM direct_messages
                     WHERE conversation_id = $1
                       AND ((created_at > $2) OR (created_at = $2 AND id > $3))\
                     ORDER BY created_at ASC, id ASC
                     LIMIT $4",
                conversation_id,
                after_time,
                after_id,
                fetch_limit
            )
            .unwrap_or_default(),
            None => db_fetch_all!(
                self,
                DirectMessageEntity,
                "SELECT * FROM direct_messages
                     WHERE conversation_id = $1
                     ORDER BY created_at ASC, id ASC
                     LIMIT $2",
                conversation_id,
                fetch_limit
            )
            .unwrap_or_default(),
        };

        let has_next_page = entities.len() > first;
        let items: Vec<DirectMessage> = entities
            .into_iter()
            .take(first)
            .map(DirectMessage::from)
            .collect();

        (items, has_next_page)
    }

    async fn edit_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
        new_content: String,
    ) -> Result<DirectMessage, DomainError> {
        let msg = db_fetch_one!(
            self,
            DirectMessageEntity,
            "UPDATE direct_messages SET content = $1, is_edited = true WHERE id = $2 AND sender_id = $3 RETURNING *",
            new_content,
            message_id,
            sender_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to edit direct message");
            DomainError::new(ErrorCode::MessageEditUnauthorized, "Cannot edit message or message not found")
        })?;

        Ok(DirectMessage::from(msg))
    }

    async fn delete_direct_message(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
    ) -> Result<bool, DomainError> {
        let rows = db_execute!(
            self,
            "DELETE FROM direct_messages WHERE id = $1 AND sender_id = $2",
            message_id,
            sender_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to delete direct message");
            DomainError::new(
                ErrorCode::MessageDeleteUnauthorized,
                "Cannot delete message",
            )
        })?;

        if rows > 0 {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::MessageNotFound,
                "Message not found or unauthorized",
            ))
        }
    }

    async fn get_user_group_conversations(&self, user_id: Uuid) -> Vec<Conversation> {
        db_fetch_all!(
            self,
            ConversationEntity,
            "SELECT c.* FROM conversations c
             JOIN conversation_participants cp ON c.id = cp.conversation_id
             WHERE cp.user_id = $1 AND c.is_group = true
             ORDER BY c.updated_at DESC",
            user_id
        )
        .unwrap_or_default()
        .into_iter()
        .map(Conversation::from)
        .collect()
    }
}
