use crate::application::AppServices;
use crate::graphql::types::CommentGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct CommentQuery;

#[Object]
impl CommentQuery {
    /// Get single comment by ID
    async fn comment(&self, ctx: &Context<'_>, id: ID) -> Result<Option<CommentGql>> {
        let services = ctx.data::<AppServices>()?;
        let cid = Uuid::parse_str(&id)?;
        let comment = services
            .comment
            .get_comment_by_id(cid)
            .await
            .map_err(|e| e.extend())?;
        Ok(comment.map(CommentGql))
    }
}
