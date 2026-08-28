pub mod mutation;
pub mod query;
pub mod subscription;
pub mod types;

use crate::application::AppServices;
use crate::infrastructure::config::AuthConfig;
use crate::infrastructure::db::loaders::{
    CommentLikesCountLoader, CommentLoader, CommentRepliesCountLoader, FollowersCountLoader,
    FollowingCountLoader, HasActiveStoriesLoader, PollOptionVotesCountLoader, PollOptionsLoader,
    PostLikesCountLoader, PostLoader, PostMediaLoader, PostPollLoader, PostRepostsCountLoader,
    UserLoader, UserPostsCountLoader,
};
use crate::infrastructure::db::postgres::Database;
use crate::infrastructure::pubsub::MessageBroker;
use async_graphql::Schema;
use async_graphql::dataloader::DataLoader;
use mutation::MutationRoot;
use query::QueryRoot;
use subscription::SubscriptionRoot;

pub type SnsSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// Build the GraphQL schema with database, DataLoaders, Application Services, pubsub broker,
/// and security protections (depth & complexity limiting, introspection control).
#[allow(dead_code)]
pub fn build_schema(db: Database, auth_config: AuthConfig, broker: MessageBroker) -> SnsSchema {
    build_schema_with_options(db, auth_config, broker, true)
}

/// Build the GraphQL schema with explicit control over introspection
pub fn build_schema_with_options(
    db: Database,
    auth_config: AuthConfig,
    broker: MessageBroker,
    enable_introspection: bool,
) -> SnsSchema {
    let app_services = AppServices::new(db.clone(), auth_config);

    // Initialize High-Performance DataLoaders for N+1 Batching
    let user_loader = DataLoader::new(UserLoader::new(db.clone()), tokio::spawn);
    let post_loader = DataLoader::new(PostLoader::new(db.clone()), tokio::spawn);
    let comment_loader = DataLoader::new(CommentLoader::new(db.clone()), tokio::spawn);
    let post_likes_loader = DataLoader::new(PostLikesCountLoader::new(db.clone()), tokio::spawn);
    let post_reposts_loader =
        DataLoader::new(PostRepostsCountLoader::new(db.clone()), tokio::spawn);
    let comment_likes_loader =
        DataLoader::new(CommentLikesCountLoader::new(db.clone()), tokio::spawn);
    let user_posts_loader = DataLoader::new(UserPostsCountLoader::new(db.clone()), tokio::spawn);
    let followers_loader = DataLoader::new(FollowersCountLoader::new(db.clone()), tokio::spawn);
    let following_loader = DataLoader::new(FollowingCountLoader::new(db.clone()), tokio::spawn);
    let active_stories_loader =
        DataLoader::new(HasActiveStoriesLoader::new(db.clone()), tokio::spawn);
    let comment_replies_count_loader =
        DataLoader::new(CommentRepliesCountLoader::new(db.clone()), tokio::spawn);
    let post_media_loader = DataLoader::new(PostMediaLoader::new(db.clone()), tokio::spawn);
    let post_poll_loader = DataLoader::new(PostPollLoader::new(db.clone()), tokio::spawn);
    let poll_options_loader = DataLoader::new(PollOptionsLoader::new(db.clone()), tokio::spawn);
    let poll_option_votes_loader =
        DataLoader::new(PollOptionVotesCountLoader::new(db.clone()), tokio::spawn);

    let mut builder = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .limit_depth(10)
        .limit_complexity(250)
        .extension(async_graphql::extensions::Tracing)
        .data(db)
        .data(app_services.auth.clone())
        .data(app_services.post.clone())
        .data(app_services.comment.clone())
        .data(app_services.social.clone())
        .data(app_services.dm.clone())
        .data(app_services.story.clone())
        .data(app_services.notification.clone())
        .data(app_services.bookmark.clone())
        .data(app_services.poll.clone())
        .data(app_services.moderation.clone())
        .data(app_services.discovery.clone())
        .data(app_services)
        .data(broker)
        .data(user_loader)
        .data(post_loader)
        .data(comment_loader)
        .data(post_likes_loader)
        .data(post_reposts_loader)
        .data(comment_likes_loader)
        .data(user_posts_loader)
        .data(followers_loader)
        .data(following_loader)
        .data(active_stories_loader)
        .data(comment_replies_count_loader)
        .data(post_media_loader)
        .data(post_poll_loader)
        .data(poll_options_loader)
        .data(poll_option_votes_loader);

    if !enable_introspection {
        builder = builder.disable_introspection();
    }

    builder.finish()
}
