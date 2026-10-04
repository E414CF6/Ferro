use super::database::Database;
use super::entities::{WikiArticleEntity, WikiRevisionEntity};
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{ArticleFilterParams, WikiArticle, WikiRevision};
use crate::domain::repositories::WikiRepository;
use crate::infrastructure::db::postgres::DatabaseBackend;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
struct ViewLogCountRow {
    article_id: Uuid,
    count: i64,
}

#[derive(Debug, Clone, FromRow)]
struct SlugRow {
    slug: String,
}

#[async_trait]
impl WikiRepository for Database {
    async fn find_articles(
        &self,
        filters: &ArticleFilterParams,
    ) -> Result<Vec<WikiArticle>, DomainError> {
        let limit = filters.limit.unwrap_or(250).clamp(1, 500) as i64;

        match self.backend() {
            DatabaseBackend::Postgres(pool) => {
                let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
                    "SELECT * FROM wiki_articles WHERE 1=1",
                );

                if let Some(ref q) = filters.q {
                    let clean_q = q.trim();
                    if !clean_q.is_empty() {
                        let pattern = format!("%{}%", clean_q.to_lowercase());
                        builder.push(" AND (LOWER(title) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(COALESCE(summary, '')) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(content) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(tags) LIKE ")
                            .push_bind(pattern)
                            .push(")");
                    }
                }

                if let Some(ref tag) = filters.tag {
                    let clean_tag = tag.trim().trim_start_matches('#');
                    if clean_tag != "전체" && !clean_tag.is_empty() {
                        let tag_pattern = format!("%\"{}\"%", clean_tag.to_lowercase());
                        builder.push(" AND LOWER(tags) LIKE ").push_bind(tag_pattern);
                    }
                }

                if let (Some(min_lat), Some(max_lat), Some(min_lng), Some(max_lng)) =
                    (filters.min_lat, filters.max_lat, filters.min_lng, filters.max_lng)
                {
                    builder.push(" AND latitude >= ").push_bind(min_lat)
                        .push(" AND latitude <= ").push_bind(max_lat)
                        .push(" AND longitude >= ").push_bind(min_lng)
                        .push(" AND longitude <= ").push_bind(max_lng);
                }

                builder.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit);

                let entities = builder
                    .build_query_as::<WikiArticleEntity>()
                    .fetch_all(pool)
                    .await
                    .map_err(|e| {
                        error!(target: "serve::wiki_repo", error = %e, "Failed to query wiki articles (postgres)");
                        DomainError::new(ErrorCode::ErrorInternal, "Failed to query wiki articles")
                    })?;

                Ok(entities.into_iter().map(WikiArticle::from).collect())
            }
            DatabaseBackend::Sqlite(pool) => {
                let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
                    "SELECT * FROM wiki_articles WHERE 1=1",
                );

                if let Some(ref q) = filters.q {
                    let clean_q = q.trim();
                    if !clean_q.is_empty() {
                        let pattern = format!("%{}%", clean_q.to_lowercase());
                        builder.push(" AND (LOWER(title) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(COALESCE(summary, '')) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(content) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(tags) LIKE ")
                            .push_bind(pattern)
                            .push(")");
                    }
                }

                if let Some(ref tag) = filters.tag {
                    let clean_tag = tag.trim().trim_start_matches('#');
                    if clean_tag != "전체" && !clean_tag.is_empty() {
                        let tag_pattern = format!("%\"{}\"%", clean_tag.to_lowercase());
                        builder.push(" AND LOWER(tags) LIKE ").push_bind(tag_pattern);
                    }
                }

                if let (Some(min_lat), Some(max_lat), Some(min_lng), Some(max_lng)) =
                    (filters.min_lat, filters.max_lat, filters.min_lng, filters.max_lng)
                {
                    builder.push(" AND latitude >= ").push_bind(min_lat)
                        .push(" AND latitude <= ").push_bind(max_lat)
                        .push(" AND longitude >= ").push_bind(min_lng)
                        .push(" AND longitude <= ").push_bind(max_lng);
                }

                builder.push(" ORDER BY updated_at DESC LIMIT ").push_bind(limit);

                let entities = builder
                    .build_query_as::<WikiArticleEntity>()
                    .fetch_all(pool)
                    .await
                    .map_err(|e| {
                        error!(target: "serve::wiki_repo", error = %e, "Failed to query wiki articles (sqlite)");
                        DomainError::new(ErrorCode::ErrorInternal, "Failed to query wiki articles")
                    })?;

                Ok(entities.into_iter().map(WikiArticle::from).collect())
            }
        }
    }

    async fn find_article_by_slug(&self, slug: &str) -> Result<Option<WikiArticle>, DomainError> {
        let entity: Option<WikiArticleEntity> = db_fetch_optional!(
            self,
            WikiArticleEntity,
            "SELECT * FROM wiki_articles WHERE slug = $1",
            slug
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, slug = %slug, "Failed to find article by slug");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to find wiki article")
        })?;

        Ok(entity.map(WikiArticle::from))
    }

    async fn find_article_by_id(&self, id: Uuid) -> Result<Option<WikiArticle>, DomainError> {
        let entity: Option<WikiArticleEntity> = db_fetch_optional!(
            self,
            WikiArticleEntity,
            "SELECT * FROM wiki_articles WHERE id = $1",
            id
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, id = %id, "Failed to find article by id");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to find wiki article")
        })?;

        Ok(entity.map(WikiArticle::from))
    }

    async fn find_slugs_by_prefix(&self, base_slug: &str) -> Result<Vec<String>, DomainError> {
        let pattern = format!("{}-%", base_slug);
        let rows: Vec<SlugRow> = db_fetch_all!(
            self,
            SlugRow,
            "SELECT slug FROM wiki_articles WHERE slug = $1 OR slug LIKE $2",
            base_slug,
            &pattern
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, "Failed to query slugs by prefix");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to check existing slugs")
        })?;

        Ok(rows.into_iter().map(|r| r.slug).collect())
    }

    async fn create_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError> {
        let tags_json = serde_json::to_string(&article.tags).unwrap_or_else(|_| "[]".to_string());

        let res = db_execute!(
            self,
            "INSERT INTO wiki_articles (id, user_id, title, slug, summary, content, latitude, longitude, zoom, category, tags, geojson, author, views, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
            article.id,
            article.user_id,
            &article.title,
            &article.slug,
            article.summary.as_deref(),
            &article.content,
            article.latitude,
            article.longitude,
            article.zoom,
            &article.category,
            &tags_json,
            article.geojson.as_deref(),
            &article.author,
            article.views,
            article.created_at,
            article.updated_at
        );

        if let Err(e) = res {
            error!(target: "serve::wiki_repo", error = %e, "Failed to insert wiki article");
            return Err(DomainError::new(
                ErrorCode::WikiArticleCreateFailed,
                "Failed to create wiki article",
            ));
        }

        let rev_res = db_execute!(
            self,
            "INSERT INTO wiki_revisions (id, article_id, user_id, title, content, latitude, longitude, edit_summary, author, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            revision.id,
            revision.article_id,
            revision.user_id,
            &revision.title,
            &revision.content,
            revision.latitude,
            revision.longitude,
            revision.edit_summary.as_deref(),
            &revision.author,
            revision.created_at
        );

        if let Err(e) = rev_res {
            error!(target: "serve::wiki_repo", error = %e, "Failed to insert initial wiki revision");
        }

        Ok(article.clone())
    }

    async fn update_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError> {
        let tags_json = serde_json::to_string(&article.tags).unwrap_or_else(|_| "[]".to_string());

        let res = db_execute!(
            self,
            "UPDATE wiki_articles
             SET title = $1, summary = $2, content = $3, latitude = $4, longitude = $5,
                 zoom = $6, category = $7, tags = $8, geojson = $9, updated_at = $10
             WHERE id = $11",
            &article.title,
            article.summary.as_deref(),
            &article.content,
            article.latitude,
            article.longitude,
            article.zoom,
            &article.category,
            &tags_json,
            article.geojson.as_deref(),
            article.updated_at,
            article.id
        );

        if let Err(e) = res {
            error!(target: "serve::wiki_repo", error = %e, "Failed to update wiki article");
            return Err(DomainError::new(
                ErrorCode::WikiArticleUpdateFailed,
                "Failed to update wiki article",
            ));
        }

        let rev_res = db_execute!(
            self,
            "INSERT INTO wiki_revisions (id, article_id, user_id, title, content, latitude, longitude, edit_summary, author, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            revision.id,
            revision.article_id,
            revision.user_id,
            &revision.title,
            &revision.content,
            revision.latitude,
            revision.longitude,
            revision.edit_summary.as_deref(),
            &revision.author,
            revision.created_at
        );

        if let Err(e) = rev_res {
            error!(target: "serve::wiki_repo", error = %e, "Failed to insert update wiki revision");
        }

        Ok(article.clone())
    }

    async fn delete_article(&self, slug: &str) -> Result<bool, DomainError> {
        let res = db_execute!(
            self,
            "DELETE FROM wiki_articles WHERE slug = $1",
            slug
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, slug = %slug, "Failed to delete wiki article");
            DomainError::new(ErrorCode::WikiArticleDeleteFailed, "Failed to delete wiki article")
        })?;

        Ok(res.rows_affected() > 0)
    }

    async fn record_view(&self, article_id: Uuid) -> Result<(), DomainError> {
        let now = Utc::now();
        let log_id = Uuid::new_v4();

        let _ = db_execute!(
            self,
            "UPDATE wiki_articles SET views = views + 1 WHERE id = $1",
            article_id
        );

        let _ = db_execute!(
            self,
            "INSERT INTO wiki_view_logs (id, article_id, created_at) VALUES ($1, $2, $3)",
            log_id,
            article_id,
            now
        );

        Ok(())
    }

    async fn get_revisions(
        &self,
        article_id: Uuid,
        limit: usize,
    ) -> Result<Vec<WikiRevision>, DomainError> {
        let lim = limit.clamp(1, 50) as i64;
        let entities: Vec<WikiRevisionEntity> = db_fetch_all!(
            self,
            WikiRevisionEntity,
            "SELECT * FROM wiki_revisions WHERE article_id = $1 ORDER BY created_at DESC LIMIT $2",
            article_id,
            lim
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, article_id = %article_id, "Failed to query wiki revisions");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to query wiki revisions")
        })?;

        Ok(entities.into_iter().map(WikiRevision::from).collect())
    }

    async fn get_all_articles_meta(&self) -> Result<Vec<WikiArticle>, DomainError> {
        let entities: Vec<WikiArticleEntity> = db_fetch_all!(
            self,
            WikiArticleEntity,
            "SELECT * FROM wiki_articles ORDER BY updated_at DESC"
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, "Failed to query all wiki articles meta");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to query wiki articles")
        })?;

        Ok(entities.into_iter().map(WikiArticle::from).collect())
    }

    async fn get_view_log_counts_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<(Uuid, i64)>, DomainError> {
        let rows: Vec<ViewLogCountRow> = db_fetch_all!(
            self,
            ViewLogCountRow,
            "SELECT article_id, COUNT(*) as count FROM wiki_view_logs WHERE created_at >= $1 GROUP BY article_id",
            since
        )
        .map_err(|e| {
            error!(target: "serve::wiki_repo", error = %e, "Failed to query view log counts");
            DomainError::new(ErrorCode::ErrorInternal, "Failed to query view logs")
        })?;

        Ok(rows.into_iter().map(|r| (r.article_id, r.count)).collect())
    }
}
