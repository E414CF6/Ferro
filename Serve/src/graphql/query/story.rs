use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::StoryRepository;
use crate::graphql::types::StoryGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct StoryQuery;

#[Object]
impl StoryQuery {
    /// Get single story by ID
    async fn story(&self, ctx: &Context<'_>, id: ID) -> Result<Option<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let sid = Uuid::parse_str(&id)?;
        Ok(db.get_story_by_id(sid).await.map(StoryGql))
    }

    /// Get all active stories for a user
    async fn stories_for_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<StoryGql>> {
        let db = ctx.data::<Database>()?;
        let uid = Uuid::parse_str(&user_id)?;
        let stories = db.get_active_stories_for_user(uid).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Stories tray/feed: returns all active stories from followed users and self
    async fn stories_feed(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Vec<StoryGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let stories = db.get_stories_feed_for_user(uid).await;
        Ok(stories.into_iter().map(StoryGql).collect())
    }
}
