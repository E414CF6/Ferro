pub mod bookmark;
pub mod comment;
pub mod dm;
pub mod moderation;
pub mod notification;
pub mod poll;
pub mod post;
pub mod social;
pub mod story;
pub mod user;

pub use bookmark::BookmarkQuery;
pub use comment::CommentQuery;
pub use dm::DmQuery;
pub use moderation::ModerationQuery;
pub use notification::NotificationQuery;
pub use poll::PollQuery;
pub use post::PostQuery;
pub use social::SocialQuery;
pub use story::StoryQuery;
pub use user::UserQuery;

use async_graphql::MergedObject;

/// Merged GraphQL query root composed of domain-specific sub-queries.
#[derive(MergedObject, Default)]
pub struct QueryRoot(
    pub UserQuery,
    pub StoryQuery,
    pub PostQuery,
    pub PollQuery,
    pub CommentQuery,
    pub NotificationQuery,
    pub DmQuery,
    pub SocialQuery,
    pub BookmarkQuery,
    pub ModerationQuery,
);
