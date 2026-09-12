use crate::domain::errors::DomainError;
use crate::domain::models::{Poll, PollOption, PollVote};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait PollRepository: Send + Sync {
    async fn create_poll(
        &self,
        post_id: Uuid,
        question: String,
        options: Vec<String>,
        duration_seconds: i64,
    ) -> Result<Poll, DomainError>;
    async fn get_poll_by_post_id(&self, post_id: Uuid) -> Option<Poll>;
    async fn get_poll_by_id(&self, poll_id: Uuid) -> Option<Poll>;
    async fn get_poll_options(&self, poll_id: Uuid) -> Vec<PollOption>;
    async fn vote_poll(
        &self,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollVote, DomainError>;
    async fn get_poll_option_votes_count(&self, option_id: Uuid) -> usize;
    async fn get_poll_total_votes(&self, poll_id: Uuid) -> usize;
    async fn get_user_vote_for_poll(&self, poll_id: Uuid, user_id: Uuid) -> Option<Uuid>;
}
