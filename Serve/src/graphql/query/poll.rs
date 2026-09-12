use crate::domain::repositories::PollRepository;
use crate::graphql::types::PollGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PollQuery;

#[Object]
impl PollQuery {
    /// Get poll details by ID
    async fn poll(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PollGql>> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&id)?;
        Ok(db.get_poll_by_id(pid).await.map(PollGql))
    }
}
