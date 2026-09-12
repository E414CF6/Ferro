use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::BookmarkRepository;
use crate::graphql::types::BookmarkCollectionGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct BookmarkMutation;

#[Object]
impl BookmarkMutation {
    /// Create a bookmark collection
    async fn create_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        is_private: Option<bool>,
        user_id: Option<ID>,
    ) -> Result<BookmarkCollectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let coll = db
            .create_bookmark_collection(uid, name, description, is_private.unwrap_or(true))
            .await
            .map_err(|e| e.extend())?;
        Ok(BookmarkCollectionGql(coll))
    }

    /// Update a bookmark collection
    async fn update_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        name: Option<String>,
        description: Option<String>,
        is_private: Option<bool>,
        user_id: Option<ID>,
    ) -> Result<BookmarkCollectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let db = ctx.data::<Database>()?;
        let coll = db
            .update_bookmark_collection(cid, uid, name, description, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(BookmarkCollectionGql(coll))
    }

    /// Delete a bookmark collection
    async fn delete_bookmark_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let db = ctx.data::<Database>()?;
        db.delete_bookmark_collection(cid, uid)
            .await
            .map_err(|e| e.extend())
    }

    /// Add a post to a bookmark collection
    async fn add_post_to_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.add_post_to_collection(cid, uid, pid)
            .await
            .map_err(|e| e.extend())
    }

    /// Remove a post from a bookmark collection
    async fn remove_post_from_collection(
        &self,
        ctx: &Context<'_>,
        collection_id: ID,
        post_id: ID,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let cid = Uuid::parse_str(&collection_id)?;
        let pid = Uuid::parse_str(&post_id)?;
        let db = ctx.data::<Database>()?;
        db.remove_post_from_collection(cid, uid, pid)
            .await
            .map_err(|e| e.extend())
    }
}
