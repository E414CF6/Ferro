use crate::domain::repositories::CommentRepository;
use crate::graphql::types::CommentGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct CommentQuery;

#[Object]
impl CommentQuery {
    /// Get single comment by ID
    async fn comment(&self, ctx: &Context<'_>, id: ID) -> Result<Option<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let cid = Uuid::parse_str(&id)?;
        Ok(db.get_comment_by_id(cid).await.map(CommentGql))
    }
}
