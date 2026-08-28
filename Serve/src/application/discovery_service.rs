#![allow(dead_code)]
use crate::domain::models::{Post, User};
use crate::domain::repositories::{PostRepository, UserRepository};
use crate::infrastructure::db::postgres::Database;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating User and Content Discovery (search, recommendations, trends)
#[derive(Clone)]
pub struct DiscoveryService {
    db: Arc<Database>,
}

impl DiscoveryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn search_users(&self, query: &str, limit: Option<usize>) -> Vec<User> {
        self.db.search_users(query, limit).await
    }

    pub async fn search_posts_cursor(
        &self,
        query: &str,
        first: usize,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        self.db.search_posts_cursor(query, first, after).await
    }

    pub async fn get_trending_hashtags(&self, limit: Option<usize>) -> Vec<(String, usize)> {
        self.db.get_trending_hashtags(limit).await
    }
}
