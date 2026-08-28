use crate::domain::errors::{DomainError, ErrorCode};
use crate::infrastructure::auth::AuthUser;

use async_graphql::{Context, ErrorExtensions, ID};
use uuid::Uuid;

/// Resolves a user ID from either an explicit ID parameter or the authenticated user context.
/// This eliminates the repeated auth resolution pattern across mutations and queries.
pub fn resolve_user_id(
    ctx: &Context<'_>,
    explicit_id: Option<ID>,
) -> Result<Uuid, async_graphql::Error> {
    if let Some(id_str) = explicit_id {
        Uuid::parse_str(&id_str).map_err(|_| {
            DomainError::new(
                ErrorCode::ErrorBadRequest,
                ErrorCode::ErrorBadRequest.as_str(),
            )
            .extend()
        })
    } else if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
        Ok(auth_user.user_id)
    } else {
        Err(DomainError::new(
            ErrorCode::AuthUserIdOrTokenRequired,
            ErrorCode::AuthUserIdOrTokenRequired.as_str(),
        )
        .extend())
    }
}

/// Requires authenticated user from context - returns error if no auth context
pub fn require_auth(ctx: &Context<'_>) -> Result<Uuid, async_graphql::Error> {
    ctx.data_opt::<AuthUser>()
        .map(|u| u.user_id)
        .ok_or_else(|| {
            DomainError::new(
                ErrorCode::AuthTokenRequired,
                ErrorCode::AuthTokenRequired.as_str(),
            )
            .extend()
        })
}
