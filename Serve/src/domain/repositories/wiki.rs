use crate::domain::errors::DomainError;
use crate::domain::models::{ArticleFilterParams, WikiArticle, WikiRevision};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait WikiRepository: Send + Sync {
    async fn find_articles(&self, filters: &ArticleFilterParams) -> Result<Vec<WikiArticle>, DomainError>;
    async fn find_article_by_slug(&self, slug: &str) -> Result<Option<WikiArticle>, DomainError>;
    async fn find_article_by_id(&self, id: Uuid) -> Result<Option<WikiArticle>, DomainError>;
    async fn find_slugs_by_prefix(&self, base_slug: &str) -> Result<Vec<String>, DomainError>;
    async fn create_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError>;
    async fn update_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError>;
    async fn delete_article(&self, slug: &str) -> Result<bool, DomainError>;
    async fn record_view(&self, article_id: Uuid) -> Result<(), DomainError>;
    async fn get_revisions(&self, article_id: Uuid, limit: usize) -> Result<Vec<WikiRevision>, DomainError>;
    async fn get_all_articles_meta(&self) -> Result<Vec<WikiArticle>, DomainError>;
    async fn get_view_log_counts_since(&self, since: DateTime<Utc>) -> Result<Vec<(Uuid, i64)>, DomainError>;
}
