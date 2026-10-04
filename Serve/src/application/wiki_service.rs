#![allow(dead_code)]
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    ArticleFilterParams, RankChangeType, TrendingArticleItem, TrendingTagItem, TrendsData,
    WikiArticle, WikiRevision,
};
use crate::domain::repositories::WikiRepository;
use crate::domain::validation::validate_wiki_article;
use crate::infrastructure::db::database::Database;

use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
struct CacheEntry {
    data: TrendsData,
    timestamp: DateTime<Utc>,
}

const CACHE_TTL_SECS: i64 = 15;
const DEFAULT_ZOOM: f64 = 15.0;
const MAX_SUMMARY_LENGTH: usize = 120;
const DEFAULT_AUTHOR: &str = "익명";

/// Application Service orchestrating Wiki operations, slug generation, revisions, and trends
#[derive(Clone)]
pub struct WikiService<R: WikiRepository = Database> {
    repo: Arc<R>,
    trends_cache: Arc<RwLock<Option<CacheEntry>>>,
}

impl<R: WikiRepository> WikiService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self {
            repo,
            trends_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn invalidate_cache(&self) {
        let mut lock = self.trends_cache.write().await;
        *lock = None;
    }

    /// URL-friendly unique slug generator with collision resolution
    pub async fn generate_unique_slug(&self, title: &str) -> Result<String, DomainError> {
        let trimmed = title.trim();
        let mut base_slug = String::new();

        for c in trimmed.chars() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                base_slug.push(c.to_ascii_lowercase());
            } else if c.is_whitespace() || "/?#[]@!$&'()*+,;=".contains(c) {
                if !base_slug.ends_with('-') && !base_slug.is_empty() {
                    base_slug.push('-');
                }
            }
        }

        let base_slug = base_slug.trim_matches('-').to_string();
        let base_slug = if base_slug.is_empty() {
            format!("article-{}", Utc::now().timestamp_millis())
        } else {
            base_slug
        };

        let existing_slugs = self.repo.find_slugs_by_prefix(&base_slug).await?;
        let existing_set: HashSet<String> = existing_slugs.into_iter().collect();

        if !existing_set.contains(&base_slug) {
            return Ok(base_slug);
        }

        let mut counter = 1;
        loop {
            let candidate = format!("{}-{}", base_slug, counter);
            if !existing_set.contains(&candidate) {
                return Ok(candidate);
            }
            counter += 1;
        }
    }

    /// Cleans and extracts hashtags
    pub fn parse_tags(tags: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for raw_tag in tags {
            let clean = raw_tag
                .trim()
                .trim_start_matches('#')
                .replace(|c: char| c.is_whitespace(), "")
                .to_string();

            if !clean.is_empty() && !seen.contains(&clean) {
                seen.insert(clean.clone());
                result.push(clean);
                if result.len() >= 15 {
                    break;
                }
            }
        }
        result
    }

    /// Auto-generates a plain summary if none provided
    pub fn generate_summary(content: &str) -> String {
        let clean: String = content
            .chars()
            .filter(|c| !matches!(*c, '#' | '*' | '`'))
            .collect();
        let clean_space = clean.split_whitespace().collect::<Vec<_>>().join(" ");

        if clean_space.chars().count() <= MAX_SUMMARY_LENGTH {
            clean_space
        } else {
            let truncated: String = clean_space.chars().take(MAX_SUMMARY_LENGTH).collect();
            format!("{}...", truncated.trim())
        }
    }

    pub async fn create_article(
        &self,
        user_id: Option<Uuid>,
        author_name: Option<String>,
        title: String,
        content: String,
        summary: Option<String>,
        latitude: f64,
        longitude: f64,
        zoom: Option<f64>,
        tags: Option<Vec<String>>,
        geojson: Option<String>,
    ) -> Result<WikiArticle, DomainError> {
        let sanitized_tags = Self::parse_tags(&tags.unwrap_or_default());
        validate_wiki_article(&title, &content, latitude, longitude, &sanitized_tags)?;

        let primary_category = sanitized_tags
            .first()
            .cloned()
            .unwrap_or_else(|| "장소".to_string());

        let slug = self.generate_unique_slug(&title).await?;
        let final_summary = summary
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| Self::generate_summary(&content));

        let author = author_name
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| DEFAULT_AUTHOR.to_string());

        let now = Utc::now();
        let article_id = Uuid::new_v4();

        let article = WikiArticle {
            id: article_id,
            user_id,
            title: title.clone(),
            slug,
            summary: Some(final_summary),
            content: content.clone(),
            latitude,
            longitude,
            zoom: zoom.unwrap_or(DEFAULT_ZOOM),
            category: primary_category,
            tags: sanitized_tags,
            geojson,
            author: author.clone(),
            views: 0,
            created_at: now,
            updated_at: now,
        };

        let revision = WikiRevision {
            id: Uuid::new_v4(),
            article_id,
            user_id,
            title,
            content,
            latitude,
            longitude,
            edit_summary: Some("새 문서 생성".to_string()),
            author,
            created_at: now,
        };

        let created = self.repo.create_article(&article, &revision).await?;
        self.invalidate_cache().await;
        Ok(created)
    }

    pub async fn update_article(
        &self,
        slug: &str,
        user_id: Option<Uuid>,
        author_name: Option<String>,
        title: Option<String>,
        content: Option<String>,
        summary: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        zoom: Option<f64>,
        tags: Option<Vec<String>>,
        geojson: Option<String>,
        edit_summary: Option<String>,
    ) -> Result<WikiArticle, DomainError> {
        let existing = self
            .repo
            .find_article_by_slug(slug)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::WikiArticleNotFound, "Wiki article not found"))?;

        let new_title = title.unwrap_or(existing.title);
        let new_content = content.unwrap_or(existing.content);
        let new_lat = latitude.unwrap_or(existing.latitude);
        let new_lng = longitude.unwrap_or(existing.longitude);
        let new_zoom = zoom.unwrap_or(existing.zoom);
        let new_tags = tags
            .map(|t| Self::parse_tags(&t))
            .unwrap_or(existing.tags);

        validate_wiki_article(&new_title, &new_content, new_lat, new_lng, &new_tags)?;

        let primary_category = new_tags
            .first()
            .cloned()
            .unwrap_or(existing.category);

        let new_summary = summary
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| Some(Self::generate_summary(&new_content)));

        let author = author_name
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .unwrap_or(existing.author);

        let now = Utc::now();
        let updated = WikiArticle {
            id: existing.id,
            user_id: user_id.or(existing.user_id),
            title: new_title.clone(),
            slug: existing.slug,
            summary: new_summary,
            content: new_content.clone(),
            latitude: new_lat,
            longitude: new_lng,
            zoom: new_zoom,
            category: primary_category,
            tags: new_tags,
            geojson: geojson.or(existing.geojson),
            author: author.clone(),
            views: existing.views,
            created_at: existing.created_at,
            updated_at: now,
        };

        let revision = WikiRevision {
            id: Uuid::new_v4(),
            article_id: existing.id,
            user_id,
            title: new_title,
            content: new_content,
            latitude: new_lat,
            longitude: new_lng,
            edit_summary: Some(edit_summary.unwrap_or_else(|| "문서 내용 수정".to_string())),
            author,
            created_at: now,
        };

        let res = self.repo.update_article(&updated, &revision).await?;
        self.invalidate_cache().await;
        Ok(res)
    }

    pub async fn delete_article(&self, slug: &str) -> Result<bool, DomainError> {
        let success = self.repo.delete_article(slug).await?;
        if success {
            self.invalidate_cache().await;
        }
        Ok(success)
    }

    pub async fn get_article_by_slug(&self, slug: &str) -> Result<Option<WikiArticle>, DomainError> {
        self.repo.find_article_by_slug(slug).await
    }

    pub async fn get_article_by_id(&self, id: Uuid) -> Result<Option<WikiArticle>, DomainError> {
        self.repo.find_article_by_id(id).await
    }

    pub async fn get_articles(
        &self,
        filters: &ArticleFilterParams,
    ) -> Result<Vec<WikiArticle>, DomainError> {
        self.repo.find_articles(filters).await
    }

    pub async fn record_view(&self, article_id: Uuid) -> Result<(), DomainError> {
        self.repo.record_view(article_id).await
    }

    pub async fn get_revisions(
        &self,
        article_id: Uuid,
        limit: usize,
    ) -> Result<Vec<WikiRevision>, DomainError> {
        self.repo.get_revisions(article_id, limit).await
    }

    /// Real-time trending calculation with 15s in-memory TTL caching
    pub async fn get_trends(&self, force_refresh: bool) -> Result<TrendsData, DomainError> {
        let now = Utc::now();

        // 1. Check in-memory cache
        if !force_refresh {
            let lock = self.trends_cache.read().await;
            if let Some(ref entry) = *lock {
                if (now - entry.timestamp).num_seconds() < CACHE_TTL_SECS {
                    return Ok(entry.data.clone());
                }
            }
        }

        // 2. Fetch all articles metadata and recent view logs
        let twenty_four_hours_ago = now - Duration::hours(24);
        let three_hours_ago = now - Duration::hours(3);

        let articles = self.repo.get_all_articles_meta().await?;

        if articles.is_empty() {
            let empty_data = TrendsData {
                articles: Vec::new(),
                tags: Vec::new(),
                updated_at: now,
                total_articles: 0,
            };
            let mut lock = self.trends_cache.write().await;
            *lock = Some(CacheEntry {
                data: empty_data.clone(),
                timestamp: now,
            });
            return Ok(empty_data);
        }

        let logs_24h = self
            .repo
            .get_view_log_counts_since(twenty_four_hours_ago)
            .await?;
        let logs_3h = self
            .repo
            .get_view_log_counts_since(three_hours_ago)
            .await?;

        let map_24h: HashMap<Uuid, i64> = logs_24h.into_iter().collect();
        let map_3h: HashMap<Uuid, i64> = logs_3h.into_iter().collect();

        // Helper struct for scored article
        struct ScoredArticle<'a> {
            article: &'a WikiArticle,
            recent_24h: i64,
            current_score: i64,
            past_score: i64,
            is_new: bool,
        }

        let mut scored_articles: Vec<ScoredArticle> = Vec::with_capacity(articles.len());

        for art in &articles {
            let recent_3h = *map_3h.get(&art.id).unwrap_or(&0);
            let recent_24h = *map_24h.get(&art.id).unwrap_or(&0);
            let past_3_to_24h = (recent_24h - recent_3h).max(0);

            let hours_since_update = (now - art.updated_at).num_seconds() as f64 / 3600.0;
            let recency_boost = (15.0 - hours_since_update.min(15.0)).max(0.0).round() as i64;

            let baseline_views = art.views;
            let current_score = recent_3h * 8
                + recent_24h * 3
                + (baseline_views as f64 * 0.8).round() as i64
                + recency_boost;

            let past_recency = if recency_boost > 5 {
                recency_boost - 5
            } else {
                0
            };
            let past_score = past_3_to_24h * 4
                + (baseline_views as f64 * 0.8).round() as i64
                + past_recency;

            let hours_since_created = (now - art.created_at).num_seconds() as f64 / 3600.0;
            let is_new = hours_since_created <= 24.0;

            scored_articles.push(ScoredArticle {
                article: art,
                recent_24h,
                current_score,
                past_score,
                is_new,
            });
        }

        // 3. Past rank calculation
        let mut past_rank_list: Vec<(Uuid, i64)> = scored_articles
            .iter()
            .map(|s| (s.article.id, s.past_score))
            .collect();
        past_rank_list.sort_by(|a, b| b.1.cmp(&a.1));
        let past_rank_map: HashMap<Uuid, i32> = past_rank_list
            .into_iter()
            .enumerate()
            .map(|(idx, (id, _))| (id, (idx + 1) as i32))
            .collect();

        // 4. Current rank calculation
        scored_articles.sort_by(|a, b| {
            if b.current_score != a.current_score {
                b.current_score.cmp(&a.current_score)
            } else {
                b.article.views.cmp(&a.article.views)
            }
        });

        // 5. Top 10 Trending Articles
        let top_articles: Vec<TrendingArticleItem> = scored_articles
            .iter()
            .take(10)
            .enumerate()
            .map(|(idx, scored)| {
                let rank = (idx + 1) as i32;
                let prev_rank = *past_rank_map.get(&scored.article.id).unwrap_or(&rank);

                let (change, change_amount) = if scored.is_new && prev_rank > 10 {
                    (RankChangeType::New, None)
                } else if prev_rank > rank {
                    (RankChangeType::Up, Some(prev_rank - rank))
                } else if prev_rank < rank {
                    (RankChangeType::Down, Some(rank - prev_rank))
                } else {
                    (RankChangeType::Same, None)
                };

                TrendingArticleItem {
                    rank,
                    prev_rank: Some(prev_rank),
                    change,
                    change_amount,
                    id: scored.article.id,
                    title: scored.article.title.clone(),
                    slug: scored.article.slug.clone(),
                    summary: scored.article.summary.clone(),
                    latitude: scored.article.latitude,
                    longitude: scored.article.longitude,
                    zoom: scored.article.zoom,
                    category: scored.article.category.clone(),
                    tags: scored.article.tags.clone(),
                    views: scored.article.views,
                    recent_views: if scored.recent_24h > 0 {
                        scored.recent_24h
                    } else {
                        scored.article.views.min(10)
                    },
                    score: scored.current_score,
                    updated_at: scored.article.updated_at,
                }
            })
            .collect();

        // 6. Real-time Trending Hashtags Top 10
        let mut tag_current: HashMap<String, (i64, i32)> = HashMap::new();
        let mut tag_past: HashMap<String, i64> = HashMap::new();

        for scored in &scored_articles {
            for tag in &scored.article.tags {
                let clean = tag.trim().trim_start_matches('#');
                if clean.is_empty() {
                    continue;
                }
                let entry = tag_current.entry(clean.to_string()).or_insert((0, 0));
                entry.0 += scored.current_score;
                entry.1 += 1;

                *tag_past.entry(clean.to_string()).or_insert(0) += scored.past_score;
            }
        }

        // Past tag ranks
        let mut past_tags: Vec<(String, i64)> = tag_past.into_iter().collect();
        past_tags.sort_by(|a, b| b.1.cmp(&a.1));
        let past_tag_rank_map: HashMap<String, i32> = past_tags
            .into_iter()
            .enumerate()
            .map(|(idx, (tag, _))| (tag, (idx + 1) as i32))
            .collect();

        // Current tag ranks
        let mut current_tags: Vec<(String, (i64, i32))> = tag_current.into_iter().collect();
        current_tags.sort_by(|a, b| {
            if b.1.0 != a.1.0 {
                b.1.0.cmp(&a.1.0)
            } else {
                b.1.1.cmp(&a.1.1)
            }
        });

        let top_tags: Vec<TrendingTagItem> = current_tags
            .into_iter()
            .take(10)
            .enumerate()
            .map(|(idx, (tag, (score, count)))| {
                let rank = (idx + 1) as i32;
                let prev_rank = past_tag_rank_map.get(&tag).copied();

                let (change, change_amount) = match prev_rank {
                    None => (RankChangeType::New, None),
                    Some(p) if p > rank => (RankChangeType::Up, Some(p - rank)),
                    Some(p) if p < rank => (RankChangeType::Down, Some(rank - p)),
                    Some(_) => (RankChangeType::Same, None),
                };

                TrendingTagItem {
                    rank,
                    prev_rank,
                    change,
                    change_amount,
                    tag,
                    count,
                    score,
                }
            })
            .collect();

        let result = TrendsData {
            articles: top_articles,
            tags: top_tags,
            updated_at: now,
            total_articles: articles.len(),
        };

        // Cache update
        let mut lock = self.trends_cache.write().await;
        *lock = Some(CacheEntry {
            data: result.clone(),
            timestamp: now,
        });

        Ok(result)
    }
}
