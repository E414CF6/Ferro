use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::graphql::types::StoryGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct StoryQuery;

#[Object]
impl StoryQuery {
    /// Get single story by ID
    async fn story(&self, ctx: &Context<'_>, id: ID) -> Result<Option<StoryGql>> {
        let services = ctx.data::<AppServices>()?;
        let sid = Uuid::parse_str(&id)?;
        let story = services
            .story
            .get_story_by_id(sid)
            .await
            .map_err(|e| e.extend())?;
        Ok(story.map(StoryGql))
    }

    /// Get all active stories for a user
    async fn stories_for_user(&self, ctx: &Context<'_>, user_id: ID) -> Result<Vec<StoryGql>> {
        let services = ctx.data::<AppServices>()?;
        let uid = Uuid::parse_str(&user_id)?;
        let stories = services
            .story
            .get_active_stories_for_user(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(stories.into_iter().map(StoryGql).collect())
    }

    /// Stories tray/feed: returns all active stories from followed users and self
    async fn stories_feed(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Vec<StoryGql>> {
        let uid = match resolve_user_id(ctx, user_id) {
            Ok(uid) => uid,
            Err(_) => return Ok(vec![]),
        };
        let services = ctx.data::<AppServices>()?;
        let stories = services
            .story
            .get_stories_feed_for_user(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(stories.into_iter().map(StoryGql).collect())
    }
}
