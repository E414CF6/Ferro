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
    #[serde(default)]
    pub jti: Option<String>,
    #[serde(default = "default_token_type")]
    pub token_type: Option<String>,
}

fn default_token_type() -> Option<String> {
    Some("access".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_in: i64,
    pub refresh_expires_in: i64,
    pub token_type: String,
    pub jti: String,
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

/// Compute SHA-256 hash of a token for secure storage in database without saving raw refresh tokens.
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Creates a Dual Token Pair: short-lived Access Token + long-lived Refresh Token with unique JTI
pub fn create_token_pair(
    user_id: Uuid,
    username: &str,
    secret: &str,
    access_minutes: i64,
    refresh_days: i64,
) -> Result<TokenPair, DomainError> {
    let now = Utc::now();
    let access_exp = now
        .checked_add_signed(Duration::minutes(access_minutes))
        .expect("valid timestamp")
        .timestamp() as usize;

    let refresh_exp = now
        .checked_add_signed(Duration::days(refresh_days))
        .expect("valid timestamp")
        .timestamp() as usize;

    let jti = Uuid::new_v4().to_string();

    let access_claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: access_exp,
        jti: Some(jti.clone()),
        token_type: Some("access".to_string()),
    };

    let refresh_claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: refresh_exp,
        jti: Some(jti.clone()),
        token_type: Some("refresh".to_string()),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    let access_token = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &access_claims,
        &encoding_key,
    )
    .map_err(|e| {
        tracing::error!(target: "serve::auth", error = %e, "Access token generation failed");
        DomainError::new(
            ErrorCode::AuthTokenGenerationFailed,
            ErrorCode::AuthTokenGenerationFailed.as_str(),
        )
    })?;

    let refresh_token = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &refresh_claims,
        &encoding_key,
    )
    .map_err(|e| {
        tracing::error!(target: "serve::auth", error = %e, "Refresh token generation failed");
        DomainError::new(
            ErrorCode::AuthTokenGenerationFailed,
            ErrorCode::AuthTokenGenerationFailed.as_str(),
        )
    })?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        access_expires_in: access_minutes * 60,
        refresh_expires_in: refresh_days * 86400,
        token_type: "Bearer".to_string(),
        jti,
    })
}

/// Legacy single JWT creator (defaults to access token type)
#[allow(dead_code)]
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
        jti: Some(Uuid::new_v4().to_string()),
        token_type: Some("access".to_string()),
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
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

/// Verifies Access JWT token (rejects tokens marked as "refresh")
pub fn verify_jwt(token: &str, secret: &str) -> Result<AuthUser, DomainError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::debug!(target: "serve::auth", error = %e, "JWT token validation failed");
        DomainError::new(
            ErrorCode::AuthTokenInvalid,
            ErrorCode::AuthTokenInvalid.as_str(),
        )
    })?;

    // Disallow using refresh token as access token
    if let Some(ref tt) = token_data.claims.token_type {
        if tt == "refresh" {
            tracing::warn!(target: "serve::auth", "Rejected refresh token presented as access token");
            return Err(DomainError::new(
                ErrorCode::AuthTokenInvalid,
                "Refresh token cannot be used for resource authentication",
            ));
        }
    }

    let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|e| {
        tracing::warn!(target: "serve::auth", error = %e, "Invalid UUID in JWT sub claim");
        DomainError::new(
            ErrorCode::AuthUserIdInvalid,
            ErrorCode::AuthUserIdInvalid.as_str(),
        )
    })?;

    Ok(AuthUser {
        user_id,
        username: token_data.claims.username,
    })
}

/// Verifies Refresh Token and extracts `(AuthUser, jti, exp)`
pub fn verify_refresh_token(
    token: &str,
    secret: &str,
) -> Result<(AuthUser, String, usize), DomainError> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::debug!(target: "serve::auth", error = %e, "Refresh token validation failed");
        DomainError::new(
            ErrorCode::AuthTokenInvalid,
            ErrorCode::AuthTokenInvalid.as_str(),
        )
    })?;

    if let Some(ref tt) = token_data.claims.token_type {
        if tt != "refresh" {
            return Err(DomainError::new(
                ErrorCode::AuthTokenInvalid,
                "Expected token of type refresh",
            ));
        }
    }

    let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|e| {
        tracing::warn!(target: "serve::auth", error = %e, "Invalid UUID in refresh token sub claim");
        DomainError::new(
            ErrorCode::AuthUserIdInvalid,
            ErrorCode::AuthUserIdInvalid.as_str(),
        )
    })?;

    let jti = token_data.claims.jti.unwrap_or_default();

    Ok((
        AuthUser {
            user_id,
            username: token_data.claims.username,
        },
        jti,
        token_data.claims.exp,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pair_and_verification() {
        let user_id = Uuid::new_v4();
        let username = "alice";
        let secret = "super-secret-key-at-least-32-chars-long!";

        let pair = create_token_pair(user_id, username, secret, 15, 30).unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());

        // Access token passes verify_jwt
        let auth_user = verify_jwt(&pair.access_token, secret).unwrap();
        assert_eq!(auth_user.user_id, user_id);
        assert_eq!(auth_user.username, username);

        // Refresh token fails verify_jwt (cannot be used as access token)
        let err = verify_jwt(&pair.refresh_token, secret);
        assert!(err.is_err());

        // Refresh token passes verify_refresh_token
        let (ref_user, jti, _) = verify_refresh_token(&pair.refresh_token, secret).unwrap();
        assert_eq!(ref_user.user_id, user_id);
        assert_eq!(jti, pair.jti);

        // Token hash is deterministic
        let hash1 = hash_token(&pair.refresh_token);
        let hash2 = hash_token(&pair.refresh_token);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }
}
