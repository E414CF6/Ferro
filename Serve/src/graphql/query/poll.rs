use crate::application::AppServices;
use crate::graphql::types::PollGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PollQuery;

#[Object]
impl PollQuery {
    /// Get poll details by ID
    async fn poll(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PollGql>> {
        let services = ctx.data::<AppServices>()?;
        let pid = Uuid::parse_str(&id)?;
        let poll = services
            .poll
            .get_poll_by_id(pid)
            .await
            .map_err(|e| e.extend())?;
        Ok(poll.map(PollGql))
    }
}
