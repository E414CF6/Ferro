use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    BookmarkCollection as BookmarkCollectionModel, Comment as CommentModel,
    Conversation as ConversationModel, ConversationSummary as ConversationSummaryModel,
    DirectMessage as DirectMessageModel, FollowRequest as FollowRequestModel,
    FollowRequestStatus as FollowRequestStatusModel, MediaType as MediaTypeModel,
    Notification as NotificationModel, NotificationType as NotificationTypeModel,
    Poll as PollModel, PollOption as PollOptionModel, Post as PostModel,
    PostAnalytics as PostAnalyticsModel, PostAudience as PostAudienceModel,
    PostMedia as PostMediaModel, Report as ReportModel, ReportReason as ReportReasonModel,
    ReportStatus as ReportStatusModel, ReportTargetType as ReportTargetTypeModel,
    Story as StoryModel, TypingEvent as TypingEventModel, User as UserModel,
    UserList as UserListModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::{
    CommentLikesCountLoader, CommentLoader, CommentRepliesCountLoader, FollowersCountLoader,
    FollowingCountLoader, HasActiveStoriesLoader, PollOptionVotesCountLoader, PollOptionsLoader,
    PostLikesCountLoader, PostLoader, PostMediaLoader, PostPollLoader, PostRepostsCountLoader,
    UserLoader, UserPostsCountLoader,
};
use crate::infrastructure::db::postgres::Database;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, InputObject, Object, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn encode_cursor(created_at: DateTime<Utc>, id: Uuid) -> String {
    let raw = format!("{}|{}", created_at.to_rfc3339(), id);
    BASE64.encode(raw)
}

pub fn decode_cursor(cursor: &str) -> Option<(DateTime<Utc>, Uuid)> {
    let bytes = BASE64.decode(cursor.trim()).ok()?;
    let s = String::from_utf8(bytes).ok()?;
    let (dt_str, id_str) = s.split_once('|').or_else(|| s.rsplit_once(':'))?;
    let dt = DateTime::parse_from_rfc3339(dt_str)
        .ok()?
        .with_timezone(&Utc);
    let id = Uuid::parse_str(id_str).ok()?;
    Some((dt, id))
}

#[derive(Clone)]
pub struct UserGql(pub UserModel);

#[Object]
impl UserGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn username(&self) -> &str {
        &self.0.username
    }

    async fn email(&self) -> &str {
        &self.0.email
    }

    async fn display_name(&self) -> &str {
        &self.0.display_name
    }

    async fn bio(&self) -> Option<&str> {
        self.0.bio.as_deref()
    }

    async fn avatar_url(&self) -> Option<&str> {
        self.0.avatar_url.as_deref()
    }

    async fn header_image_url(&self) -> Option<&str> {
        self.0.header_image_url.as_deref()
    }

    async fn location(&self) -> Option<&str> {
        self.0.location.as_deref()
    }

    async fn website(&self) -> Option<&str> {
        self.0.website.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn is_2fa_enabled(&self) -> bool {
        self.0.is_2fa_enabled
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// User posts count (batch loaded via DataLoader)
    async fn posts_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserPostsCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_user_posts_count(self.0.id).await)
        }
    }

    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_posts_by_author(self.0.id).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn active_stories(&self, ctx: &Context<'_>) -> Result<Vec<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let stories = db.get_active_stories_for_user(self.0.id).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Whether user has active stories in last 24 hours (batch loaded via DataLoader)
    async fn has_active_stories(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(loader) = ctx.data_opt::<DataLoader<HasActiveStoriesLoader>>() {
            let has = loader.load_one(self.0.id).await?.unwrap_or(false);
            Ok(has)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.has_active_stories(self.0.id).await)
        }
    }

    async fn liked_posts(&self, ctx: &Context<'_>) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_liked_posts_by_user(self.0.id).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn saved_posts(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_saved_posts(self.0.id, limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    async fn collections(&self, ctx: &Context<'_>) -> Result<Vec<BookmarkCollectionGql>> {
        let db = ctx.data::<Database>()?;
        let colls = db.get_user_collections(self.0.id).await;
        Ok(colls.into_iter().map(BookmarkCollectionGql).collect())
    }

    async fn lists(&self, ctx: &Context<'_>) -> Result<Vec<UserListGql>> {
        let db = ctx.data::<Database>()?;
        let lists = db.get_user_lists(self.0.id).await;
        Ok(lists.into_iter().map(UserListGql).collect())
    }

    async fn comments(&self, ctx: &Context<'_>) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let comments = db.get_comments_by_user(self.0.id).await;
        Ok(comments.into_iter().map(CommentGql).collect())
    }

    async fn followers(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let followers = db.get_followers(self.0.id).await;
        Ok(followers.into_iter().map(UserGql).collect())
    }

    async fn following(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let following = db.get_following(self.0.id).await;
        Ok(following.into_iter().map(UserGql).collect())
    }

    /// Followers count (batch loaded via DataLoader)
    async fn followers_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<FollowersCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_followers_count(self.0.id).await)
        }
    }

    /// Following count (batch loaded via DataLoader)
    async fn following_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<FollowingCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_following_count(self.0.id).await)
        }
    }

    async fn is_me(&self, ctx: &Context<'_>) -> bool {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id == self.0.id
        } else {
            false
        }
    }

    async fn is_followed_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_following(auth_user.user_id, self.0.id).await)
        } else {
            Ok(false)
        }
    }

    async fn is_blocking_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_blocking(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn is_blocked_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_blocking(auth_user.user_id, self.0.id).await)
        } else {
            Ok(false)
        }
    }

    async fn is_muted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_muting(auth_user.user_id, self.0.id).await)
        } else {
            Ok(false)
        }
    }

    async fn has_pending_follow_request(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db
                .has_pending_follow_request(auth_user.user_id, self.0.id)
                .await)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct StoryGql(pub StoryModel);

#[Object]
impl StoryGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn media_url(&self) -> &str {
        &self.0.media_url
    }

    async fn caption(&self) -> Option<&str> {
        self.0.caption.as_deref()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn is_expired(&self) -> bool {
        Utc::now() > self.0.expires_at
    }

    /// Story Author (batch loaded via DataLoader)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::StoryAuthorNotFound, "Story author not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::StoryAuthorNotFound, "Story author not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn views_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_story_views_count(self.0.id).await)
    }

    async fn is_viewed_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_story_viewed_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn viewers(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let auth_user = ctx.data_opt::<AuthUser>().ok_or_else(|| {
            DomainError::new(
                ErrorCode::StoryViewersAuthRequired,
                "Authentication required to view story viewers",
            )
            .extend()
        })?;

        if auth_user.user_id != self.0.author_id {
            return Err(DomainError::new(
                ErrorCode::StoryViewersUnauthorized,
                "Unauthorized: Only story authors can view viewer list",
            )
            .extend());
        }

        let db = ctx.data::<Database>()?;
        let viewers = db.get_story_viewers(self.0.id).await;
        Ok(viewers.into_iter().map(UserGql).collect())
    }
}

#[derive(Clone)]
pub struct AuthPayloadGql {
    pub token: String,
    pub user: UserGql,
}

#[Object]
impl AuthPayloadGql {
    async fn token(&self) -> &str {
        &self.token
    }

    async fn user(&self) -> &UserGql {
        &self.user
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum MediaTypeGql {
    Image,
    Video,
    Gif,
}

impl From<MediaTypeModel> for MediaTypeGql {
    fn from(m: MediaTypeModel) -> Self {
        match m {
            MediaTypeModel::Image => MediaTypeGql::Image,
            MediaTypeModel::Video => MediaTypeGql::Video,
            MediaTypeModel::Gif => MediaTypeGql::Gif,
        }
    }
}

impl From<MediaTypeGql> for MediaTypeModel {
    fn from(m: MediaTypeGql) -> Self {
        match m {
            MediaTypeGql::Image => MediaTypeModel::Image,
            MediaTypeGql::Video => MediaTypeModel::Video,
            MediaTypeGql::Gif => MediaTypeModel::Gif,
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct MediaInput {
    pub media_url: String,
    pub media_type: Option<MediaTypeGql>,
    pub alt_text: Option<String>,
    pub sort_order: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(InputObject, Clone, Debug)]
pub struct CreatePollInput {
    pub question: String,
    pub options: Vec<String>,
    pub duration_seconds: Option<i64>,
}

#[derive(Clone)]
pub struct PostMediaGql(pub PostMediaModel);

#[Object]
impl PostMediaGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn media_url(&self) -> &str {
        &self.0.media_url
    }

    async fn media_type(&self) -> MediaTypeGql {
        MediaTypeGql::from(self.0.media_type)
    }

    async fn alt_text(&self) -> Option<&str> {
        self.0.alt_text.as_deref()
    }

    async fn sort_order(&self) -> i32 {
        self.0.sort_order
    }

    async fn width(&self) -> Option<i32> {
        self.0.width
    }

    async fn height(&self) -> Option<i32> {
        self.0.height
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PostAudienceGql {
    Public,
    FollowersOnly,
    CloseFriends,
}

impl From<PostAudienceModel> for PostAudienceGql {
    fn from(a: PostAudienceModel) -> Self {
        match a {
            PostAudienceModel::Public => PostAudienceGql::Public,
            PostAudienceModel::FollowersOnly => PostAudienceGql::FollowersOnly,
            PostAudienceModel::CloseFriends => PostAudienceGql::CloseFriends,
        }
    }
}

impl From<PostAudienceGql> for PostAudienceModel {
    fn from(a: PostAudienceGql) -> Self {
        match a {
            PostAudienceGql::Public => PostAudienceModel::Public,
            PostAudienceGql::FollowersOnly => PostAudienceModel::FollowersOnly,
            PostAudienceGql::CloseFriends => PostAudienceModel::CloseFriends,
        }
    }
}

#[derive(Clone)]
pub struct PostGql(pub PostModel);

#[Object]
impl PostGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn audience(&self) -> PostAudienceGql {
        PostAudienceGql::from(self.0.audience)
    }

    async fn views_count(&self) -> i64 {
        self.0.views_count
    }

    async fn quote_post_id(&self) -> Option<ID> {
        self.0.quote_post_id.map(|id| ID(id.to_string()))
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// Post Author (batch loaded via DataLoader to eliminate N+1)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::PostAuthorNotFound, "Post author not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::PostAuthorNotFound, "Post author not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    /// Media Attachments (batch loaded via DataLoader)
    async fn media(&self, ctx: &Context<'_>) -> Result<Vec<PostMediaGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostMediaLoader>>() {
            let media_list = loader.load_one(self.0.id).await?.unwrap_or_default();
            Ok(media_list.into_iter().map(PostMediaGql).collect())
        } else {
            let db = ctx.data::<Database>()?;
            let media_list = db.get_post_media(self.0.id).await;
            Ok(media_list.into_iter().map(PostMediaGql).collect())
        }
    }

    /// Attached Poll (if any)
    async fn poll(&self, ctx: &Context<'_>) -> Result<Option<PollGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostPollLoader>>() {
            let poll_opt = loader.load_one(self.0.id).await?.flatten();
            Ok(poll_opt.map(PollGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_poll_by_post_id(self.0.id).await.map(PollGql))
        }
    }

    /// Post Analytics & Insights
    async fn analytics(&self, ctx: &Context<'_>) -> Result<PostAnalyticsGql> {
        let db = ctx.data::<Database>()?;
        let an = db
            .get_post_analytics(self.0.id)
            .await
            .map_err(|e| e.extend())?;
        Ok(PostAnalyticsGql(an))
    }

    /// Quoted Post (if this is a quote post, batch loaded via PostLoader)
    async fn quote_post(&self, ctx: &Context<'_>) -> Result<Option<PostGql>> {
        let Some(qid) = self.0.quote_post_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
            let post_opt = loader.load_one(qid).await?;
            Ok(post_opt.map(PostGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_post_by_id(qid).await.map(PostGql))
        }
    }

    /// Pinned Comment by Author (batch loaded via CommentLoader)
    async fn pinned_comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(cid) = self.0.pinned_comment_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
            let comment_opt = loader.load_one(cid).await?;
            Ok(comment_opt.map(CommentGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_by_id(cid).await.map(CommentGql))
        }
    }

    /// Likes Count (batch loaded via DataLoader)
    async fn likes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLikesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_likes_count(self.0.id).await)
        }
    }

    /// Reposts Count (batch loaded via DataLoader)
    async fn reposts_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostRepostsCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_reposts_count(self.0.id).await)
        }
    }

    async fn is_liked_by(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        let uid = if let Some(uid_str) = user_id {
            Uuid::parse_str(&uid_str)?
        } else if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id
        } else {
            return Ok(false);
        };

        Ok(db.is_post_liked_by(self.0.id, uid).await)
    }

    async fn is_reposted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_post_reposted_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn is_saved_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_post_saved_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }

    async fn hashtags(&self) -> Vec<String> {
        self.0
            .content
            .split_whitespace()
            .filter_map(|w| {
                if let Some(tag) = w.strip_prefix('#') {
                    let clean: String = tag
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !clean.is_empty() { Some(clean) } else { None }
                } else {
                    None
                }
            })
            .collect()
    }

    async fn comments(
        &self,
        ctx: &Context<'_>,
        top_level_only: Option<bool>,
    ) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let comments = if top_level_only.unwrap_or(false) {
            db.get_top_level_comments_for_post(self.0.id).await
        } else {
            db.get_comments_for_post(self.0.id).await
        };
        Ok(comments.into_iter().map(CommentGql).collect())
    }

    /// Cursor-paginated comments on this post
    async fn comments_connection(
        &self,
        ctx: &Context<'_>,
        top_level_only: Option<bool>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<CommentConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let is_top = top_level_only.unwrap_or(true);
        let (comments, has_next_page) = db
            .get_comments_cursor(self.0.id, is_top, page_size, cursor_pair)
            .await;
        let start_cursor = comments.first().map(|c| encode_cursor(c.created_at, c.id));
        let end_cursor = comments.last().map(|c| encode_cursor(c.created_at, c.id));

        let edges = comments
            .into_iter()
            .map(|c| {
                let cursor = encode_cursor(c.created_at, c.id);
                CommentEdgeGql {
                    cursor,
                    node: CommentGql(c),
                }
            })
            .collect();

        Ok(CommentConnectionGql {
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

#[derive(Clone)]
pub struct PollGql(pub PollModel);

#[Object]
impl PollGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn post_id(&self) -> ID {
        ID(self.0.post_id.to_string())
    }

    async fn question(&self) -> &str {
        &self.0.question
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn is_expired(&self) -> bool {
        Utc::now() > self.0.expires_at
    }

    async fn total_votes(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_poll_total_votes(self.0.id).await)
    }

    async fn user_voted_option_id(&self, ctx: &Context<'_>) -> Result<Option<ID>> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db
                .get_user_vote_for_poll(self.0.id, auth_user.user_id)
                .await
                .map(|id| ID(id.to_string())))
        } else {
            Ok(None)
        }
    }

    async fn options(&self, ctx: &Context<'_>) -> Result<Vec<PollOptionGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PollOptionsLoader>>() {
            let opts = loader.load_one(self.0.id).await?.unwrap_or_default();
            Ok(opts.into_iter().map(PollOptionGql).collect())
        } else {
            let db = ctx.data::<Database>()?;
            let opts = db.get_poll_options(self.0.id).await;
            Ok(opts.into_iter().map(PollOptionGql).collect())
        }
    }
}

#[derive(Clone)]
pub struct PollOptionGql(pub PollOptionModel);

#[Object]
impl PollOptionGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn option_text(&self) -> &str {
        &self.0.option_text
    }

    async fn sort_order(&self) -> i32 {
        self.0.sort_order
    }

    async fn votes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PollOptionVotesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_poll_option_votes_count(self.0.id).await)
        }
    }

    async fn percentage(&self, ctx: &Context<'_>) -> Result<f64> {
        let db = ctx.data::<Database>()?;
        let total = db.get_poll_total_votes(self.0.poll_id).await;
        if total == 0 {
            return Ok(0.0);
        }
        let count = db.get_poll_option_votes_count(self.0.id).await;
        let pct = ((count as f64) / (total as f64)) * 100.0;
        Ok((pct * 10.0).round() / 10.0)
    }

    async fn is_voted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            let user_vote = db
                .get_user_vote_for_poll(self.0.poll_id, auth_user.user_id)
                .await;
            Ok(user_vote == Some(self.0.id))
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct UserListGql(pub UserListModel);

#[Object]
impl UserListGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn owner(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.owner_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Owner not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let owner = db.get_user_by_id(self.0.owner_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Owner not found").extend()
            })?;
            Ok(UserGql(owner))
        }
    }

    async fn members(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let members = db.get_list_members(self.0.id).await;
        Ok(members.into_iter().map(UserGql).collect())
    }

    async fn members_count(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_list_members(self.0.id).await.len())
    }

    async fn feed(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db
            .get_list_feed_cursor(self.0.id, page_size, cursor_pair)
            .await;
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

#[derive(Clone)]
pub struct BookmarkCollectionGql(pub BookmarkCollectionModel);

#[Object]
impl BookmarkCollectionGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn user(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.user_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.user_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn posts_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db
            .get_collection_posts_cursor(self.0.id, page_size, cursor_pair)
            .await;
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

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportTargetTypeGql {
    Post,
    Comment,
    User,
}

impl From<ReportTargetTypeModel> for ReportTargetTypeGql {
    fn from(r: ReportTargetTypeModel) -> Self {
        match r {
            ReportTargetTypeModel::Post => ReportTargetTypeGql::Post,
            ReportTargetTypeModel::Comment => ReportTargetTypeGql::Comment,
            ReportTargetTypeModel::User => ReportTargetTypeGql::User,
        }
    }
}

impl From<ReportTargetTypeGql> for ReportTargetTypeModel {
    fn from(r: ReportTargetTypeGql) -> Self {
        match r {
            ReportTargetTypeGql::Post => ReportTargetTypeModel::Post,
            ReportTargetTypeGql::Comment => ReportTargetTypeModel::Comment,
            ReportTargetTypeGql::User => ReportTargetTypeModel::User,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportReasonGql {
    Spam,
    Harassment,
    HateSpeech,
    Inappropriate,
    Copyright,
    Other,
}

impl From<ReportReasonModel> for ReportReasonGql {
    fn from(r: ReportReasonModel) -> Self {
        match r {
            ReportReasonModel::Spam => ReportReasonGql::Spam,
            ReportReasonModel::Harassment => ReportReasonGql::Harassment,
            ReportReasonModel::HateSpeech => ReportReasonGql::HateSpeech,
            ReportReasonModel::Inappropriate => ReportReasonGql::Inappropriate,
            ReportReasonModel::Copyright => ReportReasonGql::Copyright,
            ReportReasonModel::Other => ReportReasonGql::Other,
        }
    }
}

impl From<ReportReasonGql> for ReportReasonModel {
    fn from(r: ReportReasonGql) -> Self {
        match r {
            ReportReasonGql::Spam => ReportReasonModel::Spam,
            ReportReasonGql::Harassment => ReportReasonModel::Harassment,
            ReportReasonGql::HateSpeech => ReportReasonModel::HateSpeech,
            ReportReasonGql::Inappropriate => ReportReasonModel::Inappropriate,
            ReportReasonGql::Copyright => ReportReasonModel::Copyright,
            ReportReasonGql::Other => ReportReasonModel::Other,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportStatusGql {
    Pending,
    Resolved,
    Dismissed,
}

impl From<ReportStatusModel> for ReportStatusGql {
    fn from(s: ReportStatusModel) -> Self {
        match s {
            ReportStatusModel::Pending => ReportStatusGql::Pending,
            ReportStatusModel::Resolved => ReportStatusGql::Resolved,
            ReportStatusModel::Dismissed => ReportStatusGql::Dismissed,
        }
    }
}

impl From<ReportStatusGql> for ReportStatusModel {
    fn from(s: ReportStatusGql) -> Self {
        match s {
            ReportStatusGql::Pending => ReportStatusModel::Pending,
            ReportStatusGql::Resolved => ReportStatusModel::Resolved,
            ReportStatusGql::Dismissed => ReportStatusModel::Dismissed,
        }
    }
}

#[derive(Clone)]
pub struct ReportGql(pub ReportModel);

#[Object]
impl ReportGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn target_id(&self) -> ID {
        ID(self.0.target_id.to_string())
    }

    async fn target_type(&self) -> ReportTargetTypeGql {
        ReportTargetTypeGql::from(self.0.target_type)
    }

    async fn reason(&self) -> ReportReasonGql {
        ReportReasonGql::from(self.0.reason)
    }

    async fn details(&self) -> Option<&str> {
        self.0.details.as_deref()
    }

    async fn status(&self) -> ReportStatusGql {
        ReportStatusGql::from(self.0.status)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn reporter(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db.get_user_by_id(self.0.reporter_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, "Reporter not found").extend()
        })?;
        Ok(UserGql(user))
    }
}

#[derive(Clone)]
pub struct PostAnalyticsGql(pub PostAnalyticsModel);

#[Object]
impl PostAnalyticsGql {
    async fn post_id(&self) -> ID {
        ID(self.0.post_id.to_string())
    }

    async fn views_count(&self) -> i64 {
        self.0.views_count
    }

    async fn likes_count(&self) -> usize {
        self.0.likes_count
    }

    async fn reposts_count(&self) -> usize {
        self.0.reposts_count
    }

    async fn comments_count(&self) -> usize {
        self.0.comments_count
    }

    async fn engagement_rate(&self) -> f64 {
        self.0.engagement_rate
    }
}

#[derive(Clone)]
pub struct TotpSetupGql {
    pub secret: String,
    pub otpauth_uri: String,
}

#[Object]
impl TotpSetupGql {
    async fn secret(&self) -> &str {
        &self.secret
    }

    async fn otpauth_uri(&self) -> &str {
        &self.otpauth_uri
    }
}

#[derive(Clone)]
pub struct CommentGql(pub CommentModel);

#[Object]
impl CommentGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn parent_id(&self) -> Option<ID> {
        self.0.parent_id.map(|id| ID(id.to_string()))
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn is_edited(&self) -> bool {
        self.0.is_edited
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    /// Comment Author (batch loaded via DataLoader)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::CommentAuthorNotFound, "Comment author not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::CommentAuthorNotFound, "Comment author not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn post(&self, ctx: &Context<'_>) -> Result<PostGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
            let post = loader.load_one(self.0.post_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::PostNotFound, "Post not found").extend()
            })?;
            Ok(PostGql(post))
        } else {
            let db = ctx.data::<Database>()?;
            let post = db.get_post_by_id(self.0.post_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::PostNotFound, "Post not found").extend()
            })?;
            Ok(PostGql(post))
        }
    }

    async fn parent(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(pid) = self.0.parent_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
            let comment_opt = loader.load_one(pid).await?;
            Ok(comment_opt.map(CommentGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_by_id(pid).await.map(CommentGql))
        }
    }

    /// Depth in comment thread hierarchy (0 = root top-level comment)
    async fn depth(&self, ctx: &Context<'_>) -> Result<usize> {
        let mut curr_parent = self.0.parent_id;
        let mut depth = 0;
        let db = ctx.data::<Database>()?;
        while let Some(pid) = curr_parent {
            depth += 1;
            if depth > 20 {
                break;
            }
            if let Some(parent) = db.get_comment_by_id(pid).await {
                curr_parent = parent.parent_id;
            } else {
                break;
            }
        }
        Ok(depth)
    }

    /// Root ancestor comment in conversation thread
    async fn root_comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(mut pid) = self.0.parent_id else {
            return Ok(None);
        };
        let db = ctx.data::<Database>()?;
        let mut root = None;
        let mut depth = 0;
        while depth < 20 {
            if let Some(parent) = db.get_comment_by_id(pid).await {
                if let Some(next_pid) = parent.parent_id {
                    pid = next_pid;
                    depth += 1;
                } else {
                    root = Some(parent);
                    break;
                }
            } else {
                break;
            }
        }
        Ok(root.map(CommentGql))
    }

    async fn replies(&self, ctx: &Context<'_>) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let replies = db.get_replies_for_comment(self.0.id).await;
        Ok(replies.into_iter().map(CommentGql).collect())
    }

    /// Cursor-paginated threaded replies under this comment
    async fn replies_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<CommentConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (replies, has_next_page) = db
            .get_replies_cursor(self.0.id, page_size, cursor_pair)
            .await;
        let start_cursor = replies.first().map(|c| encode_cursor(c.created_at, c.id));
        let end_cursor = replies.last().map(|c| encode_cursor(c.created_at, c.id));

        let edges = replies
            .into_iter()
            .map(|c| {
                let cursor = encode_cursor(c.created_at, c.id);
                CommentEdgeGql {
                    cursor,
                    node: CommentGql(c),
                }
            })
            .collect();

        Ok(CommentConnectionGql {
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

    async fn replies_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentRepliesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_replies_count(self.0.id).await)
        }
    }

    /// Comment Likes Count (batch loaded via DataLoader)
    async fn likes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLikesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_likes_count(self.0.id).await)
        }
    }

    /// Whether current viewer liked this comment
    async fn is_liked_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_comment_liked_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct CommentEdgeGql {
    pub cursor: String,
    pub node: CommentGql,
}

#[Object]
impl CommentEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &CommentGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct CommentConnectionGql {
    pub edges: Vec<CommentEdgeGql>,
    pub page_info: PageInfoGql,
    pub total_count: Option<usize>,
}

#[Object]
impl CommentConnectionGql {
    async fn edges(&self) -> &[CommentEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn total_count(&self) -> Option<usize> {
        self.total_count
    }
}

#[derive(Clone)]
pub struct DirectMessageGql(pub DirectMessageModel);

#[Object]
impl DirectMessageGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn conversation_id(&self) -> Option<ID> {
        self.0.conversation_id.map(|id| ID(id.to_string()))
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn is_read(&self) -> bool {
        self.0.is_read
    }

    async fn is_edited(&self) -> bool {
        self.0.is_edited
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    /// Message Sender (batch loaded via DataLoader)
    async fn sender(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.sender_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::DmSenderNotFound, "Sender user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.sender_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::DmSenderNotFound, "Sender user not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    /// Message Recipient (batch loaded via DataLoader)
    async fn recipient(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.recipient_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::DmRecipientNotFound, "Recipient user not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db
                .get_user_by_id(self.0.recipient_id)
                .await
                .ok_or_else(|| {
                    DomainError::new(ErrorCode::DmRecipientNotFound, "Recipient user not found")
                        .extend()
                })?;
            Ok(UserGql(user))
        }
    }

    async fn is_mine(&self, ctx: &Context<'_>) -> bool {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            auth_user.user_id == self.0.sender_id
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct DirectMessageEdgeGql {
    pub cursor: String,
    pub node: DirectMessageGql,
}

#[Object]
impl DirectMessageEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &DirectMessageGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct DirectMessageConnectionGql {
    pub edges: Vec<DirectMessageEdgeGql>,
    pub page_info: PageInfoGql,
}

#[Object]
impl DirectMessageConnectionGql {
    async fn edges(&self) -> &[DirectMessageEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }
}

#[derive(Clone)]
pub struct ConversationGql(pub ConversationSummaryModel);

#[Object]
impl ConversationGql {
    async fn other_user(&self) -> UserGql {
        UserGql(self.0.other_user.clone())
    }

    async fn last_message(&self) -> DirectMessageGql {
        DirectMessageGql(self.0.last_message.clone())
    }

    async fn unread_count(&self) -> i64 {
        self.0.unread_count
    }
}

#[derive(Clone)]
pub struct GroupConversationGql(pub ConversationModel);

#[Object]
impl GroupConversationGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn is_group(&self) -> bool {
        self.0.is_group
    }

    async fn title(&self) -> Option<&str> {
        self.0.title.as_deref()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    async fn creator(&self, ctx: &Context<'_>) -> Result<Option<UserGql>> {
        let Some(cid) = self.0.created_by else {
            return Ok(None);
        };
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_id(cid).await.map(UserGql))
    }

    async fn participants(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.get_conversation_participants(self.0.id).await;
        Ok(users.into_iter().map(UserGql).collect())
    }

    async fn messages(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DirectMessageGql>> {
        let db = ctx.data::<Database>()?;
        let msgs = db.get_conversation_messages(self.0.id, limit, offset).await;
        Ok(msgs.into_iter().map(DirectMessageGql).collect())
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum FollowRequestStatusGql {
    Pending,
    Accepted,
    Rejected,
}

impl From<FollowRequestStatusModel> for FollowRequestStatusGql {
    fn from(s: FollowRequestStatusModel) -> Self {
        match s {
            FollowRequestStatusModel::Pending => FollowRequestStatusGql::Pending,
            FollowRequestStatusModel::Accepted => FollowRequestStatusGql::Accepted,
            FollowRequestStatusModel::Rejected => FollowRequestStatusGql::Rejected,
        }
    }
}

#[derive(Clone)]
pub struct FollowRequestGql(pub FollowRequestModel);

#[Object]
impl FollowRequestGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn requester(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(self.0.requester_id)
            .await
            .ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Requester not found").extend()
            })?;
        Ok(UserGql(user))
    }

    async fn target(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db.get_user_by_id(self.0.target_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, "Target user not found").extend()
        })?;
        Ok(UserGql(user))
    }

    async fn status(&self) -> FollowRequestStatusGql {
        FollowRequestStatusGql::from(self.0.status)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}

#[derive(Clone, Debug)]
pub struct TypingEventGql(pub TypingEventModel);

#[Object]
impl TypingEventGql {
    async fn user_id(&self) -> ID {
        ID(self.0.user_id.to_string())
    }

    async fn conversation_id(&self) -> Option<ID> {
        self.0.conversation_id.map(|id| ID(id.to_string()))
    }

    async fn recipient_id(&self) -> Option<ID> {
        self.0.recipient_id.map(|id| ID(id.to_string()))
    }

    async fn is_typing(&self) -> bool {
        self.0.is_typing
    }
}

#[derive(Clone, Debug)]
pub struct HashtagTrendGql {
    pub tag: String,
    pub count: usize,
}

#[Object]
impl HashtagTrendGql {
    async fn tag(&self) -> &str {
        &self.tag
    }

    async fn count(&self) -> usize {
        self.count
    }
}

#[derive(Clone, Debug)]
pub struct PageInfoGql {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[Object]
impl PageInfoGql {
    async fn has_next_page(&self) -> bool {
        self.has_next_page
    }

    async fn has_previous_page(&self) -> bool {
        self.has_previous_page
    }

    async fn start_cursor(&self) -> Option<&str> {
        self.start_cursor.as_deref()
    }

    async fn end_cursor(&self) -> Option<&str> {
        self.end_cursor.as_deref()
    }
}

#[derive(Clone)]
pub struct PostEdgeGql {
    pub cursor: String,
    pub node: PostGql,
}

#[Object]
impl PostEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &PostGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct PostConnectionGql {
    pub edges: Vec<PostEdgeGql>,
    pub page_info: PageInfoGql,
    pub total_count: Option<usize>,
}

#[Object]
impl PostConnectionGql {
    async fn edges(&self) -> &[PostEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn total_count(&self) -> Option<usize> {
        self.total_count
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotificationTypeGql {
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

impl From<NotificationTypeModel> for NotificationTypeGql {
    fn from(t: NotificationTypeModel) -> Self {
        match t {
            NotificationTypeModel::LikePost => NotificationTypeGql::LikePost,
            NotificationTypeModel::CommentPost => NotificationTypeGql::CommentPost,
            NotificationTypeModel::Follow => NotificationTypeGql::Follow,
            NotificationTypeModel::ReplyComment => NotificationTypeGql::ReplyComment,
            NotificationTypeModel::Repost => NotificationTypeGql::Repost,
            NotificationTypeModel::Quote => NotificationTypeGql::Quote,
            NotificationTypeModel::Mention => NotificationTypeGql::Mention,
            NotificationTypeModel::FollowRequest => NotificationTypeGql::FollowRequest,
            NotificationTypeModel::FollowAccepted => NotificationTypeGql::FollowAccepted,
            NotificationTypeModel::LikeComment => NotificationTypeGql::LikeComment,
            NotificationTypeModel::PollEnded => NotificationTypeGql::PollEnded,
        }
    }
}

#[derive(Clone)]
pub struct NotificationGql(pub NotificationModel);

#[Object]
impl NotificationGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn r#type(&self) -> NotificationTypeGql {
        NotificationTypeGql::from(self.0.notification_type)
    }

    async fn notification_type(&self) -> NotificationTypeGql {
        NotificationTypeGql::from(self.0.notification_type)
    }

    async fn actor(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.actor_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Actor user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.actor_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Actor user not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn recipient(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.recipient_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "Recipient user not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db
                .get_user_by_id(self.0.recipient_id)
                .await
                .ok_or_else(|| {
                    DomainError::new(ErrorCode::UserNotFound, "Recipient user not found").extend()
                })?;
            Ok(UserGql(user))
        }
    }

    async fn entity_id(&self) -> Option<ID> {
        self.0.entity_id.map(|id| ID(id.to_string()))
    }

    async fn is_read(&self) -> bool {
        self.0.is_read
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn post(&self, ctx: &Context<'_>) -> Result<Option<PostGql>> {
        let Some(eid) = self.0.entity_id else {
            return Ok(None);
        };
        match self.0.notification_type {
            NotificationTypeModel::LikePost
            | NotificationTypeModel::Repost
            | NotificationTypeModel::Quote
            | NotificationTypeModel::Mention
            | NotificationTypeModel::PollEnded => {
                if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
                    let post_opt = loader.load_one(eid).await?;
                    Ok(post_opt.map(PostGql))
                } else {
                    let db = ctx.data::<Database>()?;
                    Ok(db.get_post_by_id(eid).await.map(PostGql))
                }
            }
            NotificationTypeModel::CommentPost
            | NotificationTypeModel::ReplyComment
            | NotificationTypeModel::LikeComment => {
                let comment_opt = if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>()
                {
                    loader.load_one(eid).await?
                } else {
                    let db = ctx.data::<Database>()?;
                    db.get_comment_by_id(eid).await
                };

                if let Some(comment) = comment_opt {
                    if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
                        let post_opt = loader.load_one(comment.post_id).await?;
                        Ok(post_opt.map(PostGql))
                    } else {
                        let db = ctx.data::<Database>()?;
                        Ok(db.get_post_by_id(comment.post_id).await.map(PostGql))
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    async fn comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(eid) = self.0.entity_id else {
            return Ok(None);
        };
        match self.0.notification_type {
            NotificationTypeModel::CommentPost
            | NotificationTypeModel::ReplyComment
            | NotificationTypeModel::LikeComment => {
                if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
                    let comment_opt = loader.load_one(eid).await?;
                    Ok(comment_opt.map(CommentGql))
                } else {
                    let db = ctx.data::<Database>()?;
                    Ok(db.get_comment_by_id(eid).await.map(CommentGql))
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct NotificationEdgeGql {
    pub cursor: String,
    pub node: NotificationGql,
}

#[Object]
impl NotificationEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &NotificationGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct NotificationConnectionGql {
    pub edges: Vec<NotificationEdgeGql>,
    pub page_info: PageInfoGql,
    pub unread_count: usize,
}

#[Object]
impl NotificationConnectionGql {
    async fn edges(&self) -> &[NotificationEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn unread_count(&self) -> usize {
        self.unread_count
    }
}
