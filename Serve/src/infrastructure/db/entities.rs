use crate::domain::models::*;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

/// Database row entity for the `users` table.
#[derive(Debug, Clone, FromRow)]
pub struct UserEntity {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub header_image_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    #[sqlx(default)]
    pub is_private: bool,
    #[sqlx(default)]
    pub is_2fa_enabled: bool,
    #[sqlx(default)]
    pub totp_secret: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<UserEntity> for User {
    fn from(e: UserEntity) -> Self {
        Self {
            id: e.id,
            username: e.username,
            email: e.email,
            password_hash: e.password_hash,
            display_name: e.display_name,
            bio: e.bio,
            avatar_url: e.avatar_url,
            header_image_url: e.header_image_url,
            location: e.location,
            website: e.website,
            is_private: e.is_private,
            is_2fa_enabled: e.is_2fa_enabled,
            totp_secret: e.totp_secret,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PostEntity {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    #[sqlx(default)]
    pub quote_post_id: Option<Uuid>,
    #[sqlx(default)]
    pub pinned_comment_id: Option<Uuid>,
    #[sqlx(default)]
    pub audience: String,
    #[sqlx(default)]
    pub views_count: i64,
    pub created_at: DateTime<Utc>,
}

impl From<PostEntity> for Post {
    fn from(e: PostEntity) -> Self {
        Self {
            id: e.id,
            author_id: e.author_id,
            content: e.content,
            quote_post_id: e.quote_post_id,
            pinned_comment_id: e.pinned_comment_id,
            audience: PostAudience::from_str(&e.audience).unwrap_or(PostAudience::Public),
            views_count: e.views_count,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PostMediaEntity {
    pub id: Uuid,
    pub post_id: Uuid,
    pub media_url: String,
    pub media_type: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<PostMediaEntity> for PostMedia {
    fn from(e: PostMediaEntity) -> Self {
        Self {
            id: e.id,
            post_id: e.post_id,
            media_url: e.media_url,
            media_type: MediaType::from_str(&e.media_type).unwrap_or(MediaType::Image),
            alt_text: e.alt_text,
            sort_order: e.sort_order,
            width: e.width,
            height: e.height,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PollEntity {
    pub id: Uuid,
    pub post_id: Uuid,
    pub question: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<PollEntity> for Poll {
    fn from(e: PollEntity) -> Self {
        Self {
            id: e.id,
            post_id: e.post_id,
            question: e.question,
            expires_at: e.expires_at,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PollOptionEntity {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_text: String,
    pub sort_order: i32,
}

impl From<PollOptionEntity> for PollOption {
    fn from(e: PollOptionEntity) -> Self {
        Self {
            id: e.id,
            poll_id: e.poll_id,
            option_text: e.option_text,
            sort_order: e.sort_order,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserListEntity {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
}

impl From<UserListEntity> for UserList {
    fn from(e: UserListEntity) -> Self {
        Self {
            id: e.id,
            owner_id: e.owner_id,
            name: e.name,
            description: e.description,
            is_private: e.is_private,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct BookmarkCollectionEntity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BookmarkCollectionEntity> for BookmarkCollection {
    fn from(e: BookmarkCollectionEntity) -> Self {
        Self {
            id: e.id,
            user_id: e.user_id,
            name: e.name,
            description: e.description,
            is_private: e.is_private,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ReportEntity {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ReportEntity> for Report {
    fn from(e: ReportEntity) -> Self {
        Self {
            id: e.id,
            reporter_id: e.reporter_id,
            target_type: ReportTargetType::from_str(&e.target_type)
                .unwrap_or(ReportTargetType::Post),
            target_id: e.target_id,
            reason: ReportReason::from_str(&e.reason).unwrap_or(ReportReason::Spam),
            details: e.details,
            status: ReportStatus::from_str(&e.status).unwrap_or(ReportStatus::Pending),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct CommentEntity {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    #[sqlx(default)]
    pub parent_id: Option<Uuid>,
    pub content: String,
    #[sqlx(default)]
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub updated_at: DateTime<Utc>,
}

impl From<CommentEntity> for Comment {
    fn from(e: CommentEntity) -> Self {
        Self {
            id: e.id,
            post_id: e.post_id,
            author_id: e.author_id,
            parent_id: e.parent_id,
            content: e.content,
            is_edited: e.is_edited,
            created_at: e.created_at,
            updated_at: if e.updated_at.timestamp() == 0 {
                e.created_at
            } else {
                e.updated_at
            },
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct StoryEntity {
    pub id: Uuid,
    pub author_id: Uuid,
    pub media_url: String,
    pub caption: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<StoryEntity> for Story {
    fn from(e: StoryEntity) -> Self {
        Self {
            id: e.id,
            author_id: e.author_id,
            media_url: e.media_url,
            caption: e.caption,
            created_at: e.created_at,
            expires_at: e.expires_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DirectMessageEntity {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    #[sqlx(default)]
    pub conversation_id: Option<Uuid>,
    pub content: String,
    pub is_read: bool,
    #[sqlx(default)]
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
}

impl From<DirectMessageEntity> for DirectMessage {
    fn from(e: DirectMessageEntity) -> Self {
        Self {
            id: e.id,
            sender_id: e.sender_id,
            recipient_id: e.recipient_id,
            conversation_id: e.conversation_id,
            content: e.content,
            is_read: e.is_read,
            is_edited: e.is_edited,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct ConversationEntity {
    pub id: Uuid,
    pub is_group: bool,
    pub title: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ConversationEntity> for Conversation {
    fn from(e: ConversationEntity) -> Self {
        Self {
            id: e.id,
            is_group: e.is_group,
            title: e.title,
            created_by: e.created_by,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ConversationParticipantEntity {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: DateTime<Utc>,
}

impl From<ConversationParticipantEntity> for ConversationParticipant {
    fn from(e: ConversationParticipantEntity) -> Self {
        Self {
            conversation_id: e.conversation_id,
            user_id: e.user_id,
            joined_at: e.joined_at,
            last_read_at: e.last_read_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct FollowRequestEntity {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<FollowRequestEntity> for FollowRequest {
    fn from(e: FollowRequestEntity) -> Self {
        let status =
            FollowRequestStatus::from_str(&e.status).unwrap_or(FollowRequestStatus::Pending);
        Self {
            id: e.id,
            requester_id: e.requester_id,
            target_id: e.target_id,
            status,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct HashtagEntity {
    pub id: Uuid,
    pub name: String,
    pub posts_count: i64,
    pub created_at: DateTime<Utc>,
}

impl From<HashtagEntity> for Hashtag {
    fn from(e: HashtagEntity) -> Self {
        Self {
            id: e.id,
            name: e.name,
            posts_count: e.posts_count,
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct NotificationEntity {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub actor_id: Uuid,
    pub notification_type: String,
    pub entity_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

impl From<NotificationEntity> for Notification {
    fn from(e: NotificationEntity) -> Self {
        let n_type =
            NotificationType::from_str(&e.notification_type).unwrap_or(NotificationType::LikePost);
        Self {
            id: e.id,
            recipient_id: e.recipient_id,
            actor_id: e.actor_id,
            notification_type: n_type,
            entity_id: e.entity_id,
            is_read: e.is_read,
            created_at: e.created_at,
        }
    }
}
