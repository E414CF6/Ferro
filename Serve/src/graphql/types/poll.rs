use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::models::{Poll as PollModel, PollOption as PollOptionModel};
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::{PollOptionVotesCountLoader, PollOptionsLoader};
use crate::infrastructure::db::postgres::Database;

#[derive(Clone)]
pub struct PollGql(pub PollModel);

#[Object]
impl PollGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn post_id(&self) -> ID {
        ID(self.0.post_id.to_string())
    }

    async fn question(&self) -> &str {
        &self.0.question
    }

    async fn expires_at(&self) -> DateTime<Utc> {
        self.0.expires_at
    }

    async fn is_expired(&self) -> bool {
        Utc::now() > self.0.expires_at
    }

    async fn total_votes(&self, ctx: &Context<'_>) -> Result<usize> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_poll_total_votes(self.0.id).await)
    }

    async fn user_voted_option_id(&self, ctx: &Context<'_>) -> Result<Option<ID>> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db
                .get_user_vote_for_poll(self.0.id, auth_user.user_id)
                .await
                .map(|id| ID(id.to_string())))
        } else {
            Ok(None)
        }
    }

    async fn options(&self, ctx: &Context<'_>) -> Result<Vec<PollOptionGql>> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PollOptionsLoader>>() {
            let opts = loader.load_one(self.0.id).await?.unwrap_or_default();
            Ok(opts.into_iter().map(PollOptionGql).collect())
        } else {
            let db = ctx.data::<Database>()?;
            let opts = db.get_poll_options(self.0.id).await;
            Ok(opts.into_iter().map(PollOptionGql).collect())
        }
    }
}

#[derive(Clone)]
pub struct PollOptionGql(pub PollOptionModel);

#[Object]
impl PollOptionGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn option_text(&self) -> &str {
        &self.0.option_text
    }

    async fn sort_order(&self) -> i32 {
        self.0.sort_order
    }

    async fn votes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PollOptionVotesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_poll_option_votes_count(self.0.id).await)
        }
    }

    async fn percentage(&self, ctx: &Context<'_>) -> Result<f64> {
        let db = ctx.data::<Database>()?;
        let total = db.get_poll_total_votes(self.0.poll_id).await;
        if total == 0 {
            return Ok(0.0);
        }
        let count = db.get_poll_option_votes_count(self.0.id).await;
        let pct = ((count as f64) / (total as f64)) * 100.0;
        Ok((pct * 10.0).round() / 10.0)
    }

    async fn is_voted_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            let user_vote = db
                .get_user_vote_for_poll(self.0.poll_id, auth_user.user_id)
                .await;
            Ok(user_vote == Some(self.0.id))
        } else {
            Ok(false)
        }
    }
}
