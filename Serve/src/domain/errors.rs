use async_graphql::{Error, ErrorExtensions, to_value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Standardized error codes across all domain operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Authentication & Authorization
    AuthInvalidCredentials,
    AuthTokenRequired,
    AuthTokenInvalid,
    AuthUserIdInvalid,
    AuthUserIdOrTokenRequired,
    AuthHashingFailed,
    AuthTokenGenerationFailed,
    AuthUserAlreadyExists,
    AuthInvalidUsername,
    AuthInvalidEmail,
    AuthPasswordTooShort,
    TotpInvalidCode,
    TotpAlreadyEnabled,
    TotpNotEnabled,

    // User & Follow
    UserNotFound,
    UserCannotFollowSelf,
    UserProfileUpdateFailed,
    UserFollowFailed,
    UserUnfollowFailed,

    // Trust & Safety: Blocks & Mutes
    CannotBlockSelf,
    CannotMuteSelf,
    UserBlocked,
    BlockNotFound,
    MuteNotFound,
    FollowRequestNotFound,
    FollowRequestAlreadyExists,
    FollowRequestCannotTargetSelf,

    // Story
    StoryNotFound,
    StoryAuthorNotFound,
    StoryViewersAuthRequired,
    StoryViewersUnauthorized,
    StoryCreateFailed,
    StoryDeleteFailed,
    StoryViewFailed,

    // Post & Comment & Repost
    PostNotFound,
    PostAuthorNotFound,
    PostCreateFailed,
    PostUpdateFailed,
    PostDeleteFailed,
    PostLikeFailed,
    PostUnlikeFailed,
    PostContentInvalid,
    PostAlreadyReposted,
    PostNotReposted,
    CommentNotFound,
    CommentAuthorNotFound,
    CommentCreateFailed,
    CommentDeleteFailed,
    CommentContentInvalid,
    ParentCommentNotFound,

    // Polls
    PollNotFound,
    PollAlreadyVoted,
    PollExpired,
    PollInvalidOptions,

    // Lists & Circles
    ListNotFound,
    ListUnauthorized,
    ListMemberAlreadyExists,

    // Collections
    CollectionNotFound,
    CollectionUnauthorized,

    // Reports & Moderation
    ReportNotFound,
    ReportAlreadySubmitted,

    // Direct Message & Conversations
    DmConversationsAuthRequired,
    DmMessagesAuthRequired,
    DmCannotSendToSelf,
    DmSendFailed,
    DmMarkReadFailed,
    DmSenderNotFound,
    DmRecipientNotFound,
    DmContentInvalid,
    ConversationNotFound,
    ConversationUnauthorized,
    MessageNotFound,
    MessageEditUnauthorized,
    MessageDeleteUnauthorized,

    // Hashtags & Mentions
    HashtagNotFound,

    // Notifications
    NotificationNotFound,
    NotificationMarkReadFailed,
    NotificationAuthRequired,

    // Pagination
    InvalidCursor,

    // Storage / Blob
    StorageUploadFailed,
    StorageDeleteFailed,
    StorageFileNotFound,

    // Security & Limits
    RateLimitExceeded,

    // Generic Fallbacks
    ErrorNotFound,
    ErrorBadRequest,
    ErrorUnauthenticated,
    ErrorForbidden,
    ErrorConflict,
    ErrorInternal,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::AuthInvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            ErrorCode::AuthTokenRequired => "AUTH_TOKEN_REQUIRED",
            ErrorCode::AuthTokenInvalid => "AUTH_TOKEN_INVALID",
            ErrorCode::AuthUserIdInvalid => "AUTH_USER_ID_INVALID",
            ErrorCode::AuthUserIdOrTokenRequired => "AUTH_USER_ID_OR_TOKEN_REQUIRED",
            ErrorCode::AuthHashingFailed => "AUTH_HASHING_FAILED",
            ErrorCode::AuthTokenGenerationFailed => "AUTH_TOKEN_GENERATION_FAILED",
            ErrorCode::AuthUserAlreadyExists => "AUTH_USER_ALREADY_EXISTS",
            ErrorCode::AuthInvalidUsername => "AUTH_INVALID_USERNAME",
            ErrorCode::AuthInvalidEmail => "AUTH_INVALID_EMAIL",
            ErrorCode::AuthPasswordTooShort => "AUTH_PASSWORD_TOO_SHORT",
            ErrorCode::TotpInvalidCode => "TOTP_INVALID_CODE",
            ErrorCode::TotpAlreadyEnabled => "TOTP_ALREADY_ENABLED",
            ErrorCode::TotpNotEnabled => "TOTP_NOT_ENABLED",

            ErrorCode::UserNotFound => "USER_NOT_FOUND",
            ErrorCode::UserCannotFollowSelf => "USER_CANNOT_FOLLOW_SELF",
            ErrorCode::UserProfileUpdateFailed => "USER_PROFILE_UPDATE_FAILED",
            ErrorCode::UserFollowFailed => "USER_FOLLOW_FAILED",
            ErrorCode::UserUnfollowFailed => "USER_UNFOLLOW_FAILED",

            ErrorCode::CannotBlockSelf => "CANNOT_BLOCK_SELF",
            ErrorCode::CannotMuteSelf => "CANNOT_MUTE_SELF",
            ErrorCode::UserBlocked => "USER_BLOCKED",
            ErrorCode::BlockNotFound => "BLOCK_NOT_FOUND",
            ErrorCode::MuteNotFound => "MUTE_NOT_FOUND",
            ErrorCode::FollowRequestNotFound => "FOLLOW_REQUEST_NOT_FOUND",
            ErrorCode::FollowRequestAlreadyExists => "FOLLOW_REQUEST_ALREADY_EXISTS",
            ErrorCode::FollowRequestCannotTargetSelf => "FOLLOW_REQUEST_CANNOT_TARGET_SELF",

            ErrorCode::StoryNotFound => "STORY_NOT_FOUND",
            ErrorCode::StoryAuthorNotFound => "STORY_AUTHOR_NOT_FOUND",
            ErrorCode::StoryViewersAuthRequired => "STORY_VIEWERS_AUTH_REQUIRED",
            ErrorCode::StoryViewersUnauthorized => "STORY_VIEWERS_UNAUTHORIZED",
            ErrorCode::StoryCreateFailed => "STORY_CREATE_FAILED",
            ErrorCode::StoryDeleteFailed => "STORY_DELETE_FAILED",
            ErrorCode::StoryViewFailed => "STORY_VIEW_FAILED",

            ErrorCode::PostNotFound => "POST_NOT_FOUND",
            ErrorCode::PostAuthorNotFound => "POST_AUTHOR_NOT_FOUND",
            ErrorCode::PostCreateFailed => "POST_CREATE_FAILED",
            ErrorCode::PostUpdateFailed => "POST_UPDATE_FAILED",
            ErrorCode::PostDeleteFailed => "POST_DELETE_FAILED",
            ErrorCode::PostLikeFailed => "POST_LIKE_FAILED",
            ErrorCode::PostUnlikeFailed => "POST_UNLIKE_FAILED",
            ErrorCode::PostContentInvalid => "POST_CONTENT_INVALID",
            ErrorCode::PostAlreadyReposted => "POST_ALREADY_REPOSTED",
            ErrorCode::PostNotReposted => "POST_NOT_REPOSTED",

            ErrorCode::CommentNotFound => "COMMENT_NOT_FOUND",
            ErrorCode::CommentAuthorNotFound => "COMMENT_AUTHOR_NOT_FOUND",
            ErrorCode::CommentCreateFailed => "COMMENT_CREATE_FAILED",
            ErrorCode::CommentDeleteFailed => "COMMENT_DELETE_FAILED",
            ErrorCode::CommentContentInvalid => "COMMENT_CONTENT_INVALID",
            ErrorCode::ParentCommentNotFound => "PARENT_COMMENT_NOT_FOUND",

            ErrorCode::PollNotFound => "POLL_NOT_FOUND",
            ErrorCode::PollAlreadyVoted => "POLL_ALREADY_VOTED",
            ErrorCode::PollExpired => "POLL_EXPIRED",
            ErrorCode::PollInvalidOptions => "POLL_INVALID_OPTIONS",

            ErrorCode::ListNotFound => "LIST_NOT_FOUND",
            ErrorCode::ListUnauthorized => "LIST_UNAUTHORIZED",
            ErrorCode::ListMemberAlreadyExists => "LIST_MEMBER_ALREADY_EXISTS",

            ErrorCode::CollectionNotFound => "COLLECTION_NOT_FOUND",
            ErrorCode::CollectionUnauthorized => "COLLECTION_UNAUTHORIZED",

            ErrorCode::ReportNotFound => "REPORT_NOT_FOUND",
            ErrorCode::ReportAlreadySubmitted => "REPORT_ALREADY_SUBMITTED",

            ErrorCode::DmConversationsAuthRequired => "DM_CONVERSATIONS_AUTH_REQUIRED",
            ErrorCode::DmMessagesAuthRequired => "DM_MESSAGES_AUTH_REQUIRED",
            ErrorCode::DmCannotSendToSelf => "DM_CANNOT_SEND_TO_SELF",
            ErrorCode::DmSendFailed => "DM_SEND_FAILED",
            ErrorCode::DmMarkReadFailed => "DM_MARK_READ_FAILED",
            ErrorCode::DmSenderNotFound => "DM_SENDER_NOT_FOUND",
            ErrorCode::DmRecipientNotFound => "DM_RECIPIENT_NOT_FOUND",
            ErrorCode::DmContentInvalid => "DM_CONTENT_INVALID",
            ErrorCode::ConversationNotFound => "CONVERSATION_NOT_FOUND",
            ErrorCode::ConversationUnauthorized => "CONVERSATION_UNAUTHORIZED",
            ErrorCode::MessageNotFound => "MESSAGE_NOT_FOUND",
            ErrorCode::MessageEditUnauthorized => "MESSAGE_EDIT_UNAUTHORIZED",
            ErrorCode::MessageDeleteUnauthorized => "MESSAGE_DELETE_UNAUTHORIZED",

            ErrorCode::HashtagNotFound => "HASHTAG_NOT_FOUND",

            ErrorCode::NotificationNotFound => "NOTIFICATION_NOT_FOUND",
            ErrorCode::NotificationMarkReadFailed => "NOTIFICATION_MARK_READ_FAILED",
            ErrorCode::NotificationAuthRequired => "NOTIFICATION_AUTH_REQUIRED",

            ErrorCode::InvalidCursor => "INVALID_CURSOR",

            ErrorCode::StorageUploadFailed => "STORAGE_UPLOAD_FAILED",
            ErrorCode::StorageDeleteFailed => "STORAGE_DELETE_FAILED",
            ErrorCode::StorageFileNotFound => "STORAGE_FILE_NOT_FOUND",

            ErrorCode::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",

            ErrorCode::ErrorNotFound => "NOT_FOUND",
            ErrorCode::ErrorBadRequest => "BAD_REQUEST",
            ErrorCode::ErrorUnauthenticated => "UNAUTHENTICATED",
            ErrorCode::ErrorForbidden => "FORBIDDEN",
            ErrorCode::ErrorConflict => "CONFLICT",
            ErrorCode::ErrorInternal => "INTERNAL_SERVER_ERROR",
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Rich domain error encapsulating code, human-readable message, and structured key-value parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    pub params: HashMap<String, String>,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            params: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_params<I, K, V>(code: ErrorCode, message: impl Into<String>, params: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut map = HashMap::new();
        for (k, v) in params {
            map.insert(k.into(), v.into());
        }
        Self {
            code,
            message: message.into(),
            params: map,
        }
    }
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

impl ErrorExtensions for DomainError {
    fn extend(&self) -> Error {
        let mut extensions = async_graphql::ErrorExtensionValues::default();
        extensions.set("code", self.code.as_str());
        extensions.set("detail", self.message.clone());
        if !self.params.is_empty() {
            if let Ok(val) = to_value(&self.params) {
                extensions.set("params", val);
            }
        }
        let mut err = Error::new(self.code.as_str());
        err.extensions = Some(extensions);
        err
    }
}
