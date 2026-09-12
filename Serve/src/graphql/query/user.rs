use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::UserRepository;
use crate::graphql::types::UserGql;
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Get current authenticated user (via Authorization JWT header) or specified user ID
    async fn me(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Option<UserGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_id(uid).await.map(UserGql))
    }

    /// Get user profile by ID
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<UserGql>> {
        let db = ctx.data::<Database>()?;
        let uid = Uuid::parse_str(&id)?;
        Ok(db.get_user_by_id(uid).await.map(UserGql))
    }

    /// Get user profile by username handle (e.g. "ferro_dev")
    async fn profile(&self, ctx: &Context<'_>, username: String) -> Result<Option<UserGql>> {
        let db = ctx.data::<Database>()?;
        Ok(db.get_user_by_username(&username).await.map(UserGql))
    }

    /// List all registered users
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.get_users().await;
        Ok(users.into_iter().map(UserGql).collect())
    }

    /// Search users by username or display name
    async fn search_users(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<UserGql>> {
        let db = ctx.data::<Database>()?;
        let users = db.search_users(&query, limit).await;
        Ok(users.into_iter().map(UserGql).collect())
    }
}
