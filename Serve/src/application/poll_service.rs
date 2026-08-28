#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{Poll, PollOption, PollVote};
use crate::domain::repositories::PostRepository;
use crate::infrastructure::db::postgres::Database;
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating interactive Polls, options, and voting
#[derive(Clone)]
pub struct PollService {
    db: Arc<Database>,
}

impl PollService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_poll(
        &self,
        post_id: Uuid,
        question: String,
        options: Vec<String>,
        duration_seconds: i64,
    ) -> Result<Poll, DomainError> {
        self.db
            .create_poll(post_id, question, options, duration_seconds)
            .await
    }

    pub async fn get_poll_by_post_id(&self, post_id: Uuid) -> Option<Poll> {
        self.db.get_poll_by_post_id(post_id).await
    }

    pub async fn get_poll_by_id(&self, poll_id: Uuid) -> Option<Poll> {
        self.db.get_poll_by_id(poll_id).await
    }

    pub async fn get_poll_options(&self, poll_id: Uuid) -> Vec<PollOption> {
        self.db.get_poll_options(poll_id).await
    }

    pub async fn vote_poll(
        &self,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollVote, DomainError> {
        self.db.vote_poll(poll_id, option_id, user_id).await
    }

    pub async fn get_poll_option_votes_count(&self, option_id: Uuid) -> usize {
        self.db.get_poll_option_votes_count(option_id).await
    }

    pub async fn get_poll_total_votes(&self, poll_id: Uuid) -> usize {
        self.db.get_poll_total_votes(poll_id).await
    }

    pub async fn get_user_vote_for_poll(&self, poll_id: Uuid, user_id: Uuid) -> Option<Uuid> {
        self.db.get_user_vote_for_poll(poll_id, user_id).await
    }
}
