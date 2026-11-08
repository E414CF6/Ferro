use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::graphql::types::StoryGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct StoryMutation;

#[Object]
impl StoryMutation {
    /// Create an Instagram-style Story (expires in 24 hours)
    async fn create_story(
        &self,
        ctx: &Context<'_>,
        media_url: String,
        caption: Option<String>,
        author_id: Option<ID>,
    ) -> Result<StoryGql> {
        let aid = resolve_user_id(ctx, author_id)?;
        let services = ctx.data::<AppServices>()?;
        let story = services
            .story
            .create_story(aid, media_url, caption)
            .await
            .map_err(|e| e.extend())?;
        Ok(StoryGql(story))
    }

    /// Mark a story as viewed by current user
    async fn view_story(
        &self,
        ctx: &Context<'_>,
        story_id: ID,
        viewer_id: Option<ID>,
    ) -> Result<bool> {
        let vid = resolve_user_id(ctx, viewer_id)?;
        let sid = Uuid::parse_str(&story_id)?;
        let services = ctx.data::<AppServices>()?;
        services
            .story
            .view_story(sid, vid)
            .await
            .map_err(|e| e.extend())
    }

    /// Delete a story
    async fn delete_story(
        &self,
        ctx: &Context<'_>,
        story_id: ID,
        author_id: Option<ID>,
    ) -> Result<bool> {
        let aid = resolve_user_id(ctx, author_id)?;
        let sid = Uuid::parse_str(&story_id)?;
        let services = ctx.data::<AppServices>()?;
        services
            .story
            .delete_story(sid, aid)
            .await
            .map_err(|e| e.extend())
    }
}
