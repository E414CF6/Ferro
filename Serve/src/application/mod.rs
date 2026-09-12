#![allow(dead_code)]
#![allow(unused_imports)]
pub mod auth_service;
pub mod bookmark_service;
pub mod comment_service;
pub mod discovery_service;
pub mod dm_service;
pub mod helpers;
pub mod moderation_service;
pub mod notification_service;
pub mod poll_service;
pub mod post_service;
pub mod social_service;
pub mod story_service;
pub mod totp_service;

pub use auth_service::AuthService;
pub use bookmark_service::BookmarkService;
pub use comment_service::CommentService;
pub use discovery_service::DiscoveryService;
pub use dm_service::DmService;
pub use moderation_service::ModerationService;
pub use notification_service::NotificationService;
pub use poll_service::PollService;
pub use post_service::PostService;
pub use social_service::SocialService;
pub use story_service::StoryService;

use crate::infrastructure::config::AuthConfig;
use crate::infrastructure::db::database::Database;
use std::sync::Arc;

/// Unified Application Services Container
#[derive(Clone)]
pub struct AppServices {
    pub auth: AuthService<Database>,
    pub post: PostService,
    pub comment: CommentService,
    pub social: SocialService,
    pub dm: DmService,
    pub story: StoryService,
    pub notification: NotificationService,
    pub bookmark: BookmarkService,
    pub poll: PollService,
    pub moderation: ModerationService,
    pub discovery: DiscoveryService,
}

impl AppServices {
    pub fn new(db: Database, auth_config: AuthConfig) -> Self {
        let db_arc = Arc::new(db);
        Self {
            auth: AuthService::new(db_arc.clone(), auth_config),
            post: PostService::new(db_arc.clone()),
            comment: CommentService::new(db_arc.clone()),
            social: SocialService::new(db_arc.clone()),
            dm: DmService::new(db_arc.clone()),
            story: StoryService::new(db_arc.clone()),
            notification: NotificationService::new(db_arc.clone()),
            bookmark: BookmarkService::new(db_arc.clone()),
            poll: PollService::new(db_arc.clone()),
            moderation: ModerationService::new(db_arc.clone()),
            discovery: DiscoveryService::new(db_arc),
        }
    }
}
