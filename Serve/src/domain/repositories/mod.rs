mod dm;
mod notification;
mod post;
mod story;
mod user;

pub use dm::DmRepository;
pub use notification::NotificationRepository;
pub use post::PostRepository;
pub use story::StoryRepository;
pub use user::UserRepository;

/// Unified repository super-trait combining all domain sub-repositories.
/// Any type implementing all five sub-traits automatically implements `AppRepository`.
pub trait AppRepository:
    UserRepository + PostRepository + StoryRepository + DmRepository + NotificationRepository
{
}

impl<T: UserRepository + PostRepository + StoryRepository + DmRepository + NotificationRepository>
    AppRepository for T
{
}
