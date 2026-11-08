use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::User;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::auth::{
    TokenPair, create_token_pair, hash_password, hash_token, verify_password, verify_refresh_token,
};
use crate::infrastructure::config::AuthConfig;
use crate::infrastructure::db::database::Database;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService<R: UserRepository = Database> {
    repository: Arc<R>,
    auth_config: AuthConfig,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repository: Arc<R>, auth_config: AuthConfig) -> Self {
        Self {
            repository,
            auth_config,
        }
    }

    pub async fn login(
        &self,
        username_or_email: &str,
        password: &str,
    ) -> Result<(TokenPair, User), DomainError> {
        let user = self
            .repository
            .get_user_by_identifier(username_or_email)
            .await?
            .ok_or_else(|| {
                warn!(
                    target: "serve::auth",
                    identifier = %username_or_email,
                    "Login attempt failed: user not found"
                );
                DomainError::new(
                    ErrorCode::AuthInvalidCredentials,
                    ErrorCode::AuthInvalidCredentials.as_str(),
                )
            })?;

        if !verify_password(password, &user.password_hash) {
            warn!(
                target: "serve::auth",
                user_id = %user.id,
                username = %user.username,
                "Login attempt failed: password mismatch"
            );
            return Err(DomainError::new(
                ErrorCode::AuthInvalidCredentials,
                ErrorCode::AuthInvalidCredentials.as_str(),
            ));
        }

        let pair = create_token_pair(
            user.id,
            &user.username,
            &self.auth_config.jwt_secret,
            self.auth_config.access_token_expiration_minutes,
            self.auth_config.refresh_token_expiration_days,
        )?;

        // Store hashed refresh token in database
        let token_hash = hash_token(&pair.refresh_token);
        let expires_at =
            Utc::now() + Duration::days(self.auth_config.refresh_token_expiration_days);
        let _ = self
            .repository
            .store_refresh_token(
                Uuid::new_v4(),
                user.id,
                token_hash,
                pair.jti.clone(),
                expires_at,
            )
            .await;

        info!(
            target: "serve::auth",
            user_id = %user.id,
            username = %user.username,
            "User logged in successfully (Dual Token issued)"
        );
        Ok((pair, user))
    }

    pub async fn signup(
        &self,
        username: String,
        email: String,
        password: String,
        display_name: String,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<(TokenPair, User), DomainError> {
        crate::domain::validation::validate_username(&username)?;
        crate::domain::validation::validate_email(&email)?;
        crate::domain::validation::validate_password(&password)?;

        let password_hash = hash_password(&password)?;

        let user = self
            .repository
            .register_user(
                username.clone(),
                email.clone(),
                password_hash,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await?;

        let pair = create_token_pair(
            user.id,
            &user.username,
            &self.auth_config.jwt_secret,
            self.auth_config.access_token_expiration_minutes,
            self.auth_config.refresh_token_expiration_days,
        )?;

        let token_hash = hash_token(&pair.refresh_token);
        let expires_at =
            Utc::now() + Duration::days(self.auth_config.refresh_token_expiration_days);
        let _ = self
            .repository
            .store_refresh_token(
                Uuid::new_v4(),
                user.id,
                token_hash,
                pair.jti.clone(),
                expires_at,
            )
            .await;

        info!(
            target: "serve::auth",
            user_id = %user.id,
            username = %user.username,
            email = %user.email,
            "User registered successfully (Dual Token issued)"
        );
        Ok((pair, user))
    }

    /// Rotates Refresh Token (RTR) and issues a new TokenPair.
    /// If an already revoked token is used, revokes all user sessions (token reuse attack detection).
    pub async fn refresh_tokens(
        &self,
        refresh_token_str: &str,
    ) -> Result<(TokenPair, User), DomainError> {
        let (_auth_user, _, _) =
            verify_refresh_token(refresh_token_str, &self.auth_config.jwt_secret)?;
        let token_hash = hash_token(refresh_token_str);

        let stored = self
            .repository
            .find_refresh_token(&token_hash)
            .await?
            .ok_or_else(|| {
                warn!(target: "serve::auth", "Refresh token not recognized in registry");
                DomainError::new(ErrorCode::AuthTokenInvalid, "Invalid refresh token")
            })?;

        let (_id, user_id, _jti, expires_at, revoked_at) = stored;

        if Utc::now() > expires_at {
            warn!(target: "serve::auth", user_id = %user_id, "Refresh token expired");
            return Err(DomainError::new(
                ErrorCode::AuthTokenInvalid,
                "Refresh token has expired",
            ));
        }

        // Token Reuse Detection
        if revoked_at.is_some() {
            warn!(
                target: "serve::auth::security",
                user_id = %user_id,
                "SECURITY ALERT: Revoked refresh token reuse detected! Revoking all sessions for user."
            );
            let _ = self
                .repository
                .revoke_all_user_refresh_tokens(user_id)
                .await;
            return Err(DomainError::new(
                ErrorCode::AuthTokenInvalid,
                "Session invalid. Please log in again.",
            ));
        }

        // Revoke the old token (Single-use rotation)
        let _ = self.repository.revoke_refresh_token(&token_hash).await;

        let user = self
            .repository
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found"))?;

        // Issue new token pair
        let pair = create_token_pair(
            user.id,
            &user.username,
            &self.auth_config.jwt_secret,
            self.auth_config.access_token_expiration_minutes,
            self.auth_config.refresh_token_expiration_days,
        )?;

        let new_hash = hash_token(&pair.refresh_token);
        let new_expires_at =
            Utc::now() + Duration::days(self.auth_config.refresh_token_expiration_days);
        let _ = self
            .repository
            .store_refresh_token(
                Uuid::new_v4(),
                user.id,
                new_hash,
                pair.jti.clone(),
                new_expires_at,
            )
            .await;

        info!(target: "serve::auth", user_id = %user.id, "Refresh token rotated successfully");
        Ok((pair, user))
    }

    /// Revokes current refresh token or all user sessions
    pub async fn logout(
        &self,
        user_id: Uuid,
        refresh_token_str: Option<&str>,
    ) -> Result<bool, DomainError> {
        if let Some(token) = refresh_token_str {
            let token_hash = hash_token(token);
            self.repository.revoke_refresh_token(&token_hash).await?;
        } else {
            self.repository
                .revoke_all_user_refresh_tokens(user_id)
                .await?;
        }
        info!(target: "serve::auth", user_id = %user_id, "User session(s) revoked on logout");
        Ok(true)
    }
}
