use crate::application::AppServices;
use crate::domain::models::ArticleFilterParams;
use crate::graphql::types::{ArticleFilterInput, TrendsDataGql, WikiArticleGql};
use async_graphql::{Context, ErrorExtensions, Object, Result};

#[derive(Default)]
pub struct WikiQuery;

#[Object]
impl WikiQuery {
    /// Retrieve a list of wiki articles with optional query, tag, bounding-box, and limit filters
    async fn articles(
        &self,
        ctx: &Context<'_>,
        filter: Option<ArticleFilterInput>,
    ) -> Result<Vec<WikiArticleGql>> {
        let services = ctx.data::<AppServices>()?;
        let filter_input = filter.unwrap_or_default();

        let params = ArticleFilterParams {
            q: filter_input.q,
            tag: filter_input.tag,
            min_lat: filter_input.min_lat,
            max_lat: filter_input.max_lat,
            min_lng: filter_input.min_lng,
            max_lng: filter_input.max_lng,
            limit: filter_input.limit,
        };

        let articles = services.wiki.get_articles(&params).await.map_err(|e| e.extend())?;
        Ok(articles.into_iter().map(WikiArticleGql).collect())
    }

    /// Retrieve a specific wiki article by its URL slug
    async fn article(&self, ctx: &Context<'_>, slug: String) -> Result<Option<WikiArticleGql>> {
        let services = ctx.data::<AppServices>()?;
        let article = services.wiki.get_article_by_slug(&slug).await.map_err(|e| e.extend())?;
        Ok(article.map(WikiArticleGql))
    }

    /// Retrieve real-time trending articles and hashtags with weighted ranking scores
    async fn wiki_trends(
        &self,
        ctx: &Context<'_>,
        force_refresh: Option<bool>,
    ) -> Result<TrendsDataGql> {
        let services = ctx.data::<AppServices>()?;
        let trends = services
            .wiki
            .get_trends(force_refresh.unwrap_or(false))
            .await
            .map_err(|e| e.extend())?;

        Ok(TrendsDataGql(trends))
    }
}
