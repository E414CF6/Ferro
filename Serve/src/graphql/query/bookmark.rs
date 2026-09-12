use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::BookmarkRepository;
use crate::graphql::types::BookmarkCollectionGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct BookmarkQuery;

#[Object]
impl BookmarkQuery {
    /// Get bookmark collections for a user
    async fn bookmark_collections(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
    ) -> Result<Vec<BookmarkCollectionGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let colls = db.get_user_collections(uid).await;
        Ok(colls.into_iter().map(BookmarkCollectionGql).collect())
    }

    /// Get bookmark collection by ID
    async fn bookmark_collection(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<BookmarkCollectionGql>> {
        let db = ctx.data::<Database>()?;
        let cid = Uuid::parse_str(&id)?;
        Ok(db
            .get_collection_by_id(cid)
            .await
            .map(BookmarkCollectionGql))
    }
}
