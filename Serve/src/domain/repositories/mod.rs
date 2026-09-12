mod bookmark;
mod comment;
mod dm;
mod moderation;
mod notification;
mod poll;
mod post;
mod story;
mod user;

pub use bookmark::BookmarkRepository;
pub use comment::CommentRepository;
pub use dm::DmRepository;
pub use moderation::ModerationRepository;
pub use notification::NotificationRepository;
pub use poll::PollRepository;
pub use post::PostRepository;
pub use story::StoryRepository;
pub use user::UserRepository;

/// Unified repository super-trait combining all domain sub-repositories.
/// Any type implementing all sub-traits automatically implements `AppRepository`.
pub trait AppRepository:
    UserRepository
    + PostRepository
    + CommentRepository
    + PollRepository
    + BookmarkRepository
    + ModerationRepository
    + StoryRepository
    + DmRepository
    + NotificationRepository
{
}

impl<
    T: UserRepository
        + PostRepository
        + CommentRepository
        + PollRepository
        + BookmarkRepository
        + ModerationRepository
        + StoryRepository
        + DmRepository
        + NotificationRepository,
> AppRepository for T
{
}
