use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::User;
use crate::domain::repositories::AppRepository;
use crate::infrastructure::auth::{create_jwt, hash_password, verify_password};
use crate::infrastructure::config::AuthConfig;

use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AuthService<R: AppRepository> {
    repository: Arc<R>,
    auth_config: AuthConfig,
}

impl<R: AppRepository> AuthService<R> {
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
    ) -> Result<(String, User), DomainError> {
        let user = self
            .repository
            .get_user_by_identifier(username_or_email)
            .await
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

        let token = create_jwt(
            user.id,
            &user.username,
            &self.auth_config.jwt_secret,
            self.auth_config.jwt_expiration_days,
        )?;

        info!(
            target: "serve::auth",
            user_id = %user.id,
            username = %user.username,
            "User logged in successfully"
        );
        Ok((token, user))
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
    ) -> Result<(String, User), DomainError> {
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

        let token = create_jwt(
            user.id,
            &user.username,
            &self.auth_config.jwt_secret,
            self.auth_config.jwt_expiration_days,
        )?;

        info!(
            target: "serve::auth",
            user_id = %user.id,
            username = %user.username,
            email = %user.email,
            "User registered successfully"
        );
        Ok((token, user))
    }
}
