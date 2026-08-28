use crate::domain::errors::DomainError;
use crate::domain::models::{
    BookmarkCollection, Comment, Poll, PollOption, PollVote, Post, PostAnalytics, PostAudience,
    PostMedia, Report, ReportReason, ReportStatus, ReportTargetType, User, UserList,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait PostRepository: Send + Sync {
    async fn get_user_posts_count(&self, user_id: Uuid) -> usize;
    async fn get_liked_posts_by_user(&self, user_id: Uuid) -> Vec<Post>;
    async fn get_comments_by_user(&self, user_id: Uuid) -> Vec<Comment>;
    async fn get_posts(&self, limit: Option<usize>, offset: Option<usize>) -> Vec<Post>;
    async fn get_posts_cursor(
        &self,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);
    async fn get_post_by_id(&self, id: Uuid) -> Option<Post>;
    async fn get_posts_by_author(&self, author_id: Uuid) -> Vec<Post>;
    async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError>;
    async fn create_quote_post(
        &self,
        author_id: Uuid,
        quote_post_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError>;
    async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError>;
    async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<bool, DomainError>;
    async fn get_feed(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Post>;
    async fn get_feed_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);
    async fn search_posts(&self, query: &str) -> Vec<Post>;
    async fn search_posts_cursor(
        &self,
        query: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);
    async fn get_posts_by_hashtag_cursor(
        &self,
        hashtag: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);

    // Comments
    async fn create_comment(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
    ) -> Result<Comment, DomainError>;
    async fn edit_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
        new_content: String,
    ) -> Result<Comment, DomainError>;
    async fn delete_comment(&self, comment_id: Uuid, author_id: Uuid) -> Result<bool, DomainError>;
    async fn get_comment_by_id(&self, comment_id: Uuid) -> Option<Comment>;
    async fn get_comments_for_post(&self, post_id: Uuid) -> Vec<Comment>;
    async fn get_top_level_comments_for_post(&self, post_id: Uuid) -> Vec<Comment>;
    async fn get_comments_cursor(
        &self,
        post_id: Uuid,
        top_level_only: bool,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool);
    async fn get_replies_for_comment(&self, comment_id: Uuid) -> Vec<Comment>;
    async fn get_replies_cursor(
        &self,
        comment_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool);
    async fn get_replies_count(&self, comment_id: Uuid) -> usize;
    async fn like_comment(&self, user_id: Uuid, comment_id: Uuid) -> Result<Comment, DomainError>;
    async fn unlike_comment(&self, user_id: Uuid, comment_id: Uuid)
    -> Result<Comment, DomainError>;
    async fn get_comment_likes_count(&self, comment_id: Uuid) -> usize;
    async fn is_comment_liked_by(&self, comment_id: Uuid, user_id: Uuid) -> bool;
    async fn pin_comment(
        &self,
        post_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    ) -> Result<Post, DomainError>;
    async fn unpin_comment(&self, post_id: Uuid, author_id: Uuid) -> Result<Post, DomainError>;
    async fn get_pinned_comment(&self, post_id: Uuid) -> Option<Comment>;

    // Likes & Reposts & Bookmarks
    async fn like_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unlike_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn get_likes_count(&self, post_id: Uuid) -> usize;
    async fn is_post_liked_by(&self, post_id: Uuid, user_id: Uuid) -> bool;
    async fn repost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unrepost_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn get_reposts_count(&self, post_id: Uuid) -> usize;
    async fn is_post_reposted_by(&self, post_id: Uuid, user_id: Uuid) -> bool;
    async fn save_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn unsave_post(&self, user_id: Uuid, post_id: Uuid) -> Result<Post, DomainError>;
    async fn is_post_saved_by(&self, post_id: Uuid, user_id: Uuid) -> bool;
    async fn get_saved_posts(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<Post>;
    async fn get_saved_posts_cursor(
        &self,
        user_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);
    async fn get_trending_hashtags(&self, limit: Option<usize>) -> Vec<(String, usize)>;

    // 1. Media Attachments
    async fn add_post_media(
        &self,
        post_id: Uuid,
        media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError>;
    async fn get_post_media(&self, post_id: Uuid) -> Vec<PostMedia>;

    // 2. Polls & Voting
    async fn create_poll(
        &self,
        post_id: Uuid,
        question: String,
        options: Vec<String>,
        duration_seconds: i64,
    ) -> Result<Poll, DomainError>;
    async fn get_poll_by_post_id(&self, post_id: Uuid) -> Option<Poll>;
    async fn get_poll_by_id(&self, poll_id: Uuid) -> Option<Poll>;
    async fn get_poll_options(&self, poll_id: Uuid) -> Vec<PollOption>;
    async fn vote_poll(
        &self,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollVote, DomainError>;
    async fn get_poll_option_votes_count(&self, option_id: Uuid) -> usize;
    async fn get_poll_total_votes(&self, poll_id: Uuid) -> usize;
    async fn get_user_vote_for_poll(&self, poll_id: Uuid, user_id: Uuid) -> Option<Uuid>;

    // 3. User Lists & Custom Feeds
    async fn create_user_list(
        &self,
        owner_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
        member_ids: Vec<Uuid>,
    ) -> Result<UserList, DomainError>;
    async fn update_user_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<UserList, DomainError>;
    async fn delete_user_list(&self, list_id: Uuid, owner_id: Uuid) -> Result<bool, DomainError>;
    async fn add_user_to_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn remove_user_from_list(
        &self,
        list_id: Uuid,
        owner_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn get_user_lists(&self, user_id: Uuid) -> Vec<UserList>;
    async fn get_user_list_by_id(&self, list_id: Uuid) -> Option<UserList>;
    async fn get_list_members(&self, list_id: Uuid) -> Vec<User>;
    async fn get_list_feed_cursor(
        &self,
        list_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);

    // 4. Bookmark Collections
    async fn create_bookmark_collection(
        &self,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        is_private: bool,
    ) -> Result<BookmarkCollection, DomainError>;
    async fn update_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
    ) -> Result<BookmarkCollection, DomainError>;
    async fn delete_bookmark_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn add_post_to_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn remove_post_from_collection(
        &self,
        collection_id: Uuid,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<bool, DomainError>;
    async fn get_user_collections(&self, user_id: Uuid) -> Vec<BookmarkCollection>;
    async fn get_collection_by_id(&self, collection_id: Uuid) -> Option<BookmarkCollection>;
    async fn get_collection_posts_cursor(
        &self,
        collection_id: Uuid,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool);

    // 5. Reports & Moderation
    async fn create_report(
        &self,
        reporter_id: Uuid,
        target_type: ReportTargetType,
        target_id: Uuid,
        reason: ReportReason,
        details: Option<String>,
    ) -> Result<Report, DomainError>;
    async fn resolve_report(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Report, DomainError>;
    async fn get_reports(&self, status: Option<ReportStatus>, limit: Option<usize>) -> Vec<Report>;

    // 6. Post Views & Analytics
    async fn record_post_view(&self, post_id: Uuid, viewer_id: Uuid) -> Result<bool, DomainError>;
    async fn get_post_analytics(&self, post_id: Uuid) -> Result<PostAnalytics, DomainError>;
}
