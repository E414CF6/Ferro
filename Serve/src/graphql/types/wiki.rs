use crate::domain::models::{
    RankChangeType, TrendingArticleItem, TrendingTagItem, TrendsData, WikiArticle, WikiRevision,
};
use crate::infrastructure::db::loaders::WikiRevisionsLoader;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ID, InputObject, Object, Result};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct WikiArticleGql(pub WikiArticle);

#[Object]
impl WikiArticleGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn user_id(&self) -> Option<ID> {
        self.0.user_id.map(|u| ID(u.to_string()))
    }

    async fn title(&self) -> &str {
        &self.0.title
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn summary(&self) -> Option<&str> {
        self.0.summary.as_deref()
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn latitude(&self) -> f64 {
        self.0.latitude
    }

    async fn longitude(&self) -> f64 {
        self.0.longitude
    }

    async fn zoom(&self) -> f64 {
        self.0.zoom
    }

    async fn category(&self) -> &str {
        &self.0.category
    }

    async fn tags(&self) -> &[String] {
        &self.0.tags
    }

    async fn geojson(&self) -> Option<&str> {
        self.0.geojson.as_deref()
    }

    async fn author(&self) -> &str {
        &self.0.author
    }

    async fn views(&self) -> i64 {
        self.0.views
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    /// Fetches revisions for this article via DataLoader
    async fn revisions(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
    ) -> Result<Vec<WikiRevisionGql>> {
        let loader = ctx.data::<DataLoader<WikiRevisionsLoader>>()?;
        let revisions = loader.load_one(self.0.id).await?.unwrap_or_default();
        let take_limit = limit.unwrap_or(15).clamp(1, 50);

        Ok(revisions
            .into_iter()
            .take(take_limit)
            .map(WikiRevisionGql)
            .collect())
    }
}

#[derive(Clone)]
pub struct WikiRevisionGql(pub WikiRevision);

#[Object]
impl WikiRevisionGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn article_id(&self) -> ID {
        ID(self.0.article_id.to_string())
    }

    async fn user_id(&self) -> Option<ID> {
        self.0.user_id.map(|u| ID(u.to_string()))
    }

    async fn title(&self) -> &str {
        &self.0.title
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn latitude(&self) -> f64 {
        self.0.latitude
    }

    async fn longitude(&self) -> f64 {
        self.0.longitude
    }

    async fn edit_summary(&self) -> Option<&str> {
        self.0.edit_summary.as_deref()
    }

    async fn author(&self) -> &str {
        &self.0.author
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }
}

#[derive(Clone)]
pub struct TrendingArticleItemGql(pub TrendingArticleItem);

#[Object]
impl TrendingArticleItemGql {
    async fn rank(&self) -> i32 {
        self.0.rank
    }

    async fn prev_rank(&self) -> Option<i32> {
        self.0.prev_rank
    }

    async fn change(&self) -> RankChangeType {
        self.0.change
    }

    async fn change_amount(&self) -> Option<i32> {
        self.0.change_amount
    }

    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn title(&self) -> &str {
        &self.0.title
    }

    async fn slug(&self) -> &str {
        &self.0.slug
    }

    async fn summary(&self) -> Option<&str> {
        self.0.summary.as_deref()
    }

    async fn latitude(&self) -> f64 {
        self.0.latitude
    }

    async fn longitude(&self) -> f64 {
        self.0.longitude
    }

    async fn zoom(&self) -> f64 {
        self.0.zoom
    }

    async fn category(&self) -> &str {
        &self.0.category
    }

    async fn tags(&self) -> &[String] {
        &self.0.tags
    }

    async fn views(&self) -> i64 {
        self.0.views
    }

    async fn recent_views(&self) -> i64 {
        self.0.recent_views
    }

    async fn score(&self) -> i64 {
        self.0.score
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }
}

#[derive(Clone)]
pub struct TrendingTagItemGql(pub TrendingTagItem);

#[Object]
impl TrendingTagItemGql {
    async fn rank(&self) -> i32 {
        self.0.rank
    }

    async fn prev_rank(&self) -> Option<i32> {
        self.0.prev_rank
    }

    async fn change(&self) -> RankChangeType {
        self.0.change
    }

    async fn change_amount(&self) -> Option<i32> {
        self.0.change_amount
    }

    async fn tag(&self) -> &str {
        &self.0.tag
    }

    async fn count(&self) -> i32 {
        self.0.count
    }

    async fn score(&self) -> i64 {
        self.0.score
    }
}

#[derive(Clone)]
pub struct TrendsDataGql(pub TrendsData);

#[Object]
impl TrendsDataGql {
    async fn articles(&self) -> Vec<TrendingArticleItemGql> {
        self.0.articles.iter().cloned().map(TrendingArticleItemGql).collect()
    }

    async fn tags(&self) -> Vec<TrendingTagItemGql> {
        self.0.tags.iter().cloned().map(TrendingTagItemGql).collect()
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    async fn total_articles(&self) -> usize {
        self.0.total_articles
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct CreateArticleInput {
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub zoom: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub geojson: Option<String>,
}

#[derive(InputObject, Clone, Debug)]
pub struct UpdateArticleInput {
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub zoom: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub geojson: Option<String>,
    pub edit_summary: Option<String>,
}

#[derive(InputObject, Clone, Debug, Default)]
pub struct ArticleFilterInput {
    pub q: Option<String>,
    pub tag: Option<String>,
    pub min_lat: Option<f64>,
    pub max_lat: Option<f64>,
    pub min_lng: Option<f64>,
    pub max_lng: Option<f64>,
    pub limit: Option<usize>,
}
