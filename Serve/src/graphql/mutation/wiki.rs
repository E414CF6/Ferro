use crate::application::AppServices;
use crate::graphql::types::{CreateArticleInput, UpdateArticleInput, WikiArticleGql};
use crate::infrastructure::auth::AuthUser;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct WikiMutation;

#[Object]
impl WikiMutation {
    /// Create a new wiki article with automatic slug generation, initial revision, and optional auth linking
    async fn create_article(
        &self,
        ctx: &Context<'_>,
        input: CreateArticleInput,
    ) -> Result<WikiArticleGql> {
        let services = ctx.data::<AppServices>()?;
        let auth_user = ctx.data_opt::<AuthUser>();

        let user_id = auth_user.map(|u| u.user_id);
        let author_name = input.author.or_else(|| {
            auth_user.map(|u| u.username.clone())
        });

        let article = services
            .wiki
            .create_article(
                user_id,
                author_name,
                input.title,
                input.content,
                input.summary,
                input.latitude,
                input.longitude,
                input.zoom,
                input.tags,
                input.geojson,
            )
            .await
            .map_err(|e| e.extend())?;

        Ok(WikiArticleGql(article))
    }

    /// Update an existing wiki article and create a new revision entry
    async fn update_article(
        &self,
        ctx: &Context<'_>,
        slug: String,
        input: UpdateArticleInput,
    ) -> Result<WikiArticleGql> {
        let services = ctx.data::<AppServices>()?;
        let auth_user = ctx.data_opt::<AuthUser>();

        let user_id = auth_user.map(|u| u.user_id);
        let author_name = input.author.or_else(|| {
            auth_user.map(|u| u.username.clone())
        });

        let updated = services
            .wiki
            .update_article(
                &slug,
                user_id,
                author_name,
                input.title,
                input.content,
                input.summary,
                input.latitude,
                input.longitude,
                input.zoom,
                input.tags,
                input.geojson,
                input.edit_summary,
            )
            .await
            .map_err(|e| e.extend())?;

        Ok(WikiArticleGql(updated))
    }

    /// Delete a wiki article by its URL slug
    async fn delete_article(&self, ctx: &Context<'_>, slug: String) -> Result<bool> {
        let services = ctx.data::<AppServices>()?;
        let success = services
            .wiki
            .delete_article(&slug)
            .await
            .map_err(|e| e.extend())?;

        Ok(success)
    }

    /// Increment view count and log view event for trending analytics
    async fn record_article_view(&self, ctx: &Context<'_>, article_id: ID) -> Result<bool> {
        let services = ctx.data::<AppServices>()?;
        let aid = Uuid::parse_str(&article_id)?;
        services
            .wiki
            .record_view(aid)
            .await
            .map_err(|e| e.extend())?;

        Ok(true)
    }
}
