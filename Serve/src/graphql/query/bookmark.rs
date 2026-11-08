use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::graphql::types::BookmarkCollectionGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
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
        let services = ctx.data::<AppServices>()?;
        let colls = services
            .bookmark
            .get_user_collections(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(colls.into_iter().map(BookmarkCollectionGql).collect())
    }

    /// Get bookmark collection by ID
    async fn bookmark_collection(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Option<BookmarkCollectionGql>> {
        let services = ctx.data::<AppServices>()?;
        let cid = Uuid::parse_str(&id)?;
        let col = services
            .bookmark
            .get_collection_by_id(cid)
            .await
            .map_err(|e| e.extend())?;
        Ok(col.map(BookmarkCollectionGql))
    }
}
