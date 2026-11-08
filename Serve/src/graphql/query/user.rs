use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::graphql::types::UserGql;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// Get current authenticated user (via Authorization JWT header) or specified user ID
    async fn me(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Option<UserGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let user = services
            .social
            .get_user_by_id(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(user.map(UserGql))
    }

    /// Get user profile by ID
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<UserGql>> {
        let services = ctx.data::<AppServices>()?;
        let uid = Uuid::parse_str(&id)?;
        let user = services
            .social
            .get_user_by_id(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(user.map(UserGql))
    }

    /// Get user profile by username handle (e.g. "ferro_dev")
    async fn profile(&self, ctx: &Context<'_>, username: String) -> Result<Option<UserGql>> {
        let services = ctx.data::<AppServices>()?;
        let user = services
            .social
            .get_user_by_username(&username)
            .await
            .map_err(|e| e.extend())?;
        Ok(user.map(UserGql))
    }

    /// List all registered users
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<UserGql>> {
        let services = ctx.data::<AppServices>()?;
        let users = services.social.get_users().await.map_err(|e| e.extend())?;
        Ok(users.into_iter().map(UserGql).collect())
    }

    /// Search users by username or display name
    async fn search_users(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<UserGql>> {
        let services = ctx.data::<AppServices>()?;
        let users = services
            .discovery
            .search_users(&query, limit)
            .await
            .map_err(|e| e.extend())?;
        Ok(users.into_iter().map(UserGql).collect())
    }
}
