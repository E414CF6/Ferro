use crate::domain::errors::{DomainError, ErrorCode};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

pub fn hash_password(password: &str) -> Result<String, DomainError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            tracing::error!(target: "serve::auth", error = %e, "Password hashing failed");
            DomainError::new(
                ErrorCode::AuthHashingFailed,
                ErrorCode::AuthHashingFailed.as_str(),
            )
        })
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_jwt(
    user_id: Uuid,
    username: &str,
    secret: &str,
    expiration_days: i64,
) -> Result<String, DomainError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(expiration_days))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!(target: "serve::auth", error = %e, "JWT token generation failed");
        DomainError::new(
            ErrorCode::AuthTokenGenerationFailed,
            ErrorCode::AuthTokenGenerationFailed.as_str(),
        )
    })
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<AuthUser, DomainError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| {
        tracing::debug!(target: "serve::auth", error = %e, "JWT token validation failed");
        DomainError::new(
            ErrorCode::AuthTokenInvalid,
            ErrorCode::AuthTokenInvalid.as_str(),
        )
    })?;

    let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|e| {
        tracing::warn!(target: "serve::auth", error = %e, "Invalid UUID in JWT sub claim");
        DomainError::new(
            ErrorCode::AuthUserIdInvalid,
            ErrorCode::AuthUserIdInvalid.as_str(),
        )
    })?;

    tracing::debug!(
        target: "serve::auth",
        user_id = %user_id,
        username = %token_data.claims.username,
        "JWT token verified successfully"
    );

    Ok(AuthUser {
        user_id,
        username: token_data.claims.username,
    })
}
