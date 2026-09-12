pub mod auth;
pub mod bookmark;
pub mod comment;
pub mod dm;
pub mod moderation;
pub mod notification;
pub mod poll;
pub mod post;
pub mod social;
pub mod story;

pub use auth::AuthMutation;
pub use bookmark::BookmarkMutation;
pub use comment::CommentMutation;
pub use dm::DmMutation;
pub use moderation::ModerationMutation;
pub use notification::NotificationMutation;
pub use poll::PollMutation;
pub use post::PostMutation;
pub use social::SocialMutation;
pub use story::StoryMutation;

use async_graphql::MergedObject;

/// Merged GraphQL mutation root composed of domain-specific sub-mutations.
#[derive(MergedObject, Default)]
pub struct MutationRoot(
    pub AuthMutation,
    pub SocialMutation,
    pub StoryMutation,
    pub PostMutation,
    pub PollMutation,
    pub CommentMutation,
    pub BookmarkMutation,
    pub ModerationMutation,
    pub DmMutation,
    pub NotificationMutation,
);
