use crate::application::helpers::resolve_user_id;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::repositories::PollRepository;
use crate::graphql::types::PollOptionGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PollMutation;

#[Object]
impl PollMutation {
    /// Vote on a poll option
    async fn vote_poll(
        &self,
        ctx: &Context<'_>,
        poll_id: ID,
        option_id: ID,
        user_id: Option<ID>,
    ) -> Result<PollOptionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let pid = Uuid::parse_str(&poll_id)?;
        let oid = Uuid::parse_str(&option_id)?;
        let db = ctx.data::<Database>()?;
        let _ = db.vote_poll(pid, oid, uid).await.map_err(|e| e.extend())?;

        let options = db.get_poll_options(pid).await;
        let opt = options.into_iter().find(|o| o.id == oid).ok_or_else(|| {
            DomainError::new(ErrorCode::ErrorNotFound, "Option not found").extend()
        })?;

        Ok(PollOptionGql(opt))
    }
}
