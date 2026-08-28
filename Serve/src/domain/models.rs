use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
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
    pub is_private: bool,
    pub is_2fa_enabled: bool,
    pub totp_secret: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PostAudience {
    Public,
    FollowersOnly,
    CloseFriends,
}

impl PostAudience {
    pub fn as_str(&self) -> &'static str {
        match self {
            PostAudience::Public => "PUBLIC",
            PostAudience::FollowersOnly => "FOLLOWERS_ONLY",
            PostAudience::CloseFriends => "CLOSE_FRIENDS",
        }
    }
}

impl std::str::FromStr for PostAudience {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "FOLLOWERS_ONLY" => PostAudience::FollowersOnly,
            "CLOSE_FRIENDS" => PostAudience::CloseFriends,
            _ => PostAudience::Public,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub quote_post_id: Option<Uuid>,
    pub pinned_comment_id: Option<Uuid>,
    pub audience: PostAudience,
    pub views_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Image,
    Video,
    Gif,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Image => "IMAGE",
            MediaType::Video => "VIDEO",
            MediaType::Gif => "GIF",
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "VIDEO" => MediaType::Video,
            "GIF" => MediaType::Gif,
            _ => MediaType::Image,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMedia {
    pub id: Uuid,
    pub post_id: Uuid,
    pub media_url: String,
    pub media_type: MediaType,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: Uuid,
    pub post_id: Uuid,
    pub question: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_text: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollVote {
    pub poll_id: Uuid,
    pub option_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserList {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkCollection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportTargetType {
    Post,
    Comment,
    User,
}

impl ReportTargetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportTargetType::Post => "POST",
            ReportTargetType::Comment => "COMMENT",
            ReportTargetType::User => "USER",
        }
    }
}

impl std::str::FromStr for ReportTargetType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "COMMENT" => ReportTargetType::Comment,
            "USER" => ReportTargetType::User,
            _ => ReportTargetType::Post,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportReason {
    Spam,
    Harassment,
    HateSpeech,
    Inappropriate,
    Copyright,
    Other,
}

impl ReportReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportReason::Spam => "SPAM",
            ReportReason::Harassment => "HARASSMENT",
            ReportReason::HateSpeech => "HATE_SPEECH",
            ReportReason::Inappropriate => "INAPPROPRIATE",
            ReportReason::Copyright => "COPYRIGHT",
            ReportReason::Other => "OTHER",
        }
    }
}

impl std::str::FromStr for ReportReason {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HARASSMENT" => ReportReason::Harassment,
            "HATE_SPEECH" => ReportReason::HateSpeech,
            "INAPPROPRIATE" => ReportReason::Inappropriate,
            "COPYRIGHT" => ReportReason::Copyright,
            "OTHER" => ReportReason::Other,
            _ => ReportReason::Spam,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReportStatus {
    Pending,
    Resolved,
    Dismissed,
}

impl ReportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportStatus::Pending => "PENDING",
            ReportStatus::Resolved => "RESOLVED",
            ReportStatus::Dismissed => "DISMISSED",
        }
    }
}

impl std::str::FromStr for ReportStatus {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "RESOLVED" => ReportStatus::Resolved,
            "DISMISSED" => ReportStatus::Dismissed,
            _ => ReportStatus::Pending,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub target_type: ReportTargetType,
    pub target_id: Uuid,
    pub reason: ReportReason,
    pub details: Option<String>,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostAnalytics {
    pub post_id: Uuid,
    pub views_count: i64,
    pub likes_count: usize,
    pub reposts_count: usize,
    pub comments_count: usize,
    pub engagement_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Follow {
    pub follower_id: Uuid,
    pub followee_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Like {
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct CommentLike {
    pub user_id: Uuid,
    pub comment_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Repost {
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: Uuid,
    pub author_id: Uuid,
    pub media_url: String,
    pub caption: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct StoryView {
    pub story_id: Uuid,
    pub viewer_id: Uuid,
    pub viewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub content: String,
    pub is_read: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    pub is_group: bool,
    pub title: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ConversationParticipant {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub other_user: User,
    pub last_message: DirectMessage,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Block {
    pub blocker_id: Uuid,
    pub blocked_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Mute {
    pub muter_id: Uuid,
    pub muted_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FollowRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

impl FollowRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FollowRequestStatus::Pending => "PENDING",
            FollowRequestStatus::Accepted => "ACCEPTED",
            FollowRequestStatus::Rejected => "REJECTED",
        }
    }
}

impl std::str::FromStr for FollowRequestStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(FollowRequestStatus::Pending),
            "ACCEPTED" => Ok(FollowRequestStatus::Accepted),
            "REJECTED" => Ok(FollowRequestStatus::Rejected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowRequest {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub target_id: Uuid,
    pub status: FollowRequestStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Hashtag {
    pub id: Uuid,
    pub name: String,
    pub posts_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingEvent {
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
    pub is_typing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationType {
    LikePost,
    CommentPost,
    Follow,
    ReplyComment,
    Repost,
    Quote,
    Mention,
    FollowRequest,
    FollowAccepted,
    LikeComment,
    PollEnded,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::LikePost => "LIKE_POST",
            NotificationType::CommentPost => "COMMENT_POST",
            NotificationType::Follow => "FOLLOW",
            NotificationType::ReplyComment => "REPLY_COMMENT",
            NotificationType::Repost => "REPOST",
            NotificationType::Quote => "QUOTE",
            NotificationType::Mention => "MENTION",
            NotificationType::FollowRequest => "FOLLOW_REQUEST",
            NotificationType::FollowAccepted => "FOLLOW_ACCEPTED",
            NotificationType::LikeComment => "LIKE_COMMENT",
            NotificationType::PollEnded => "POLL_ENDED",
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LIKE_POST" => Ok(NotificationType::LikePost),
            "COMMENT_POST" => Ok(NotificationType::CommentPost),
            "FOLLOW" => Ok(NotificationType::Follow),
            "REPLY_COMMENT" => Ok(NotificationType::ReplyComment),
            "REPOST" => Ok(NotificationType::Repost),
            "QUOTE" => Ok(NotificationType::Quote),
            "MENTION" => Ok(NotificationType::Mention),
            "FOLLOW_REQUEST" => Ok(NotificationType::FollowRequest),
            "FOLLOW_ACCEPTED" => Ok(NotificationType::FollowAccepted),
            "LIKE_COMMENT" => Ok(NotificationType::LikeComment),
            "POLL_ENDED" => Ok(NotificationType::PollEnded),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub actor_id: Uuid,
    pub notification_type: NotificationType,
    pub entity_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}
