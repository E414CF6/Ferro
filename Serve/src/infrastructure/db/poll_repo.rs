use super::database::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Poll, PollOption, PollVote};
use crate::domain::repositories::PollRepository;
use crate::infrastructure::db::entities::{PollEntity, PollOptionEntity};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl PollRepository for Database {
    async fn create_poll(
        &self,
        post_id: Uuid,
        question: String,
        options: Vec<String>,
        duration_seconds: i64,
    ) -> Result<Poll, DomainError> {
        let poll_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(duration_seconds);

        db_execute!(
            self,
            "INSERT INTO polls (id, post_id, question, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)",
            poll_id,
            post_id,
            &question,
            expires_at,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create poll");
            DomainError::new(ErrorCode::PollInvalidOptions, "Failed to create poll")
        })?;

        for (idx, opt_text) in options.iter().enumerate() {
            let opt_id = Uuid::new_v4();
            db_execute!(
                self,
                "INSERT INTO poll_options (id, poll_id, option_text, sort_order) VALUES ($1, $2, $3, $4)",
                opt_id,
                poll_id,
                opt_text,
                idx as i32
            )
            .map_err(|e| {
                error!(target: "serve::db", error = %e, "Failed to insert poll option");
                DomainError::new(ErrorCode::PollInvalidOptions, "Failed to create poll option")
            })?;
        }

        Ok(Poll {
            id: poll_id,
            post_id,
            question,
            expires_at,
            created_at: now,
        })
    }

    async fn get_poll_by_post_id(&self, post_id: Uuid) -> Option<Poll> {
        let entity: Option<PollEntity> = db_fetch_optional!(
            self,
            PollEntity,
            "SELECT * FROM polls WHERE post_id = $1",
            post_id
        )
        .ok()
        .flatten();

        entity.map(Poll::from)
    }

    async fn get_poll_by_id(&self, poll_id: Uuid) -> Option<Poll> {
        let entity: Option<PollEntity> = db_fetch_optional!(
            self,
            PollEntity,
            "SELECT * FROM polls WHERE id = $1",
            poll_id
        )
        .ok()
        .flatten();

        entity.map(Poll::from)
    }

    async fn get_poll_options(&self, poll_id: Uuid) -> Vec<PollOption> {
        let entities: Vec<PollOptionEntity> = db_fetch_all!(
            self,
            PollOptionEntity,
            "SELECT * FROM poll_options WHERE poll_id = $1 ORDER BY sort_order ASC",
            poll_id
        )
        .unwrap_or_default();

        entities.into_iter().map(PollOption::from).collect()
    }

    async fn vote_poll(
        &self,
        poll_id: Uuid,
        option_id: Uuid,
        user_id: Uuid,
    ) -> Result<PollVote, DomainError> {
        let poll = self
            .get_poll_by_id(poll_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::PollNotFound, "Poll not found"))?;

        if Utc::now() > poll.expires_at {
            return Err(DomainError::new(ErrorCode::PollExpired, "Poll has expired"));
        }

        let now = Utc::now();
        let res = db_execute!(
            self,
            "INSERT INTO poll_votes (poll_id, option_id, user_id, created_at) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
            poll_id,
            option_id,
            user_id,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to cast vote on poll");
            DomainError::new(ErrorCode::PollAlreadyVoted, "Failed to cast vote")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::PollAlreadyVoted,
                "User has already voted in this poll",
            ));
        }

        Ok(PollVote {
            poll_id,
            option_id,
            user_id,
            created_at: now,
        })
    }

    async fn get_poll_option_votes_count(&self, option_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM poll_votes WHERE option_id = $1",
            option_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_poll_total_votes(&self, poll_id: Uuid) -> usize {
        db_scalar!(
            self,
            i64,
            "SELECT COUNT(*) FROM poll_votes WHERE poll_id = $1",
            poll_id
        )
        .unwrap_or(0) as usize
    }

    async fn get_user_vote_for_poll(&self, poll_id: Uuid, user_id: Uuid) -> Option<Uuid> {
        db_scalar!(
            self,
            Uuid,
            "SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2",
            poll_id,
            user_id
        )
        .ok()
    }
}
