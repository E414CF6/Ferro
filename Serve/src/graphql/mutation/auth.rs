use crate::application::auth_service::AuthService;
use crate::application::helpers::resolve_user_id;
use crate::application::totp_service::TotpService;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::repositories::UserRepository;
use crate::graphql::types::{AuthPayloadGql, TotpSetupGql, UserGql};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Register a new user with full profile fields and return JWT auth token
    async fn signup(
        &self,
        ctx: &Context<'_>,
        username: String,
        email: String,
        password: String,
        display_name: String,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<AuthPayloadGql> {
        let auth_service = ctx.data::<AuthService<Database>>()?;
        let (token, user) = auth_service
            .signup(
                username,
                email,
                password,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token,
            user: UserGql(user),
        })
    }

    /// Login user with username/email and password
    async fn login(
        &self,
        ctx: &Context<'_>,
        username_or_email: String,
        password: String,
    ) -> Result<AuthPayloadGql> {
        let auth_service = ctx.data::<AuthService<Database>>()?;
        let (token, user) = auth_service
            .login(&username_or_email, &password)
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token,
            user: UserGql(user),
        })
    }

    /// Update user profile details
    async fn update_user_profile(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<UserGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .update_user_profile(
                uid,
                display_name,
                bio,
                avatar_url,
                header_image_url,
                location,
                website,
            )
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(user))
    }

    /// Update account privacy setting (public vs private account)
    async fn update_user_privacy(
        &self,
        ctx: &Context<'_>,
        is_private: bool,
        user_id: Option<ID>,
    ) -> Result<UserGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .update_user_privacy(uid, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(user))
    }

    /// Generate TOTP secret and QR/auth URI for 2FA setup
    async fn setup_2fa(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<TotpSetupGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        if user.is_2fa_enabled {
            return Err(
                DomainError::new(ErrorCode::TotpAlreadyEnabled, "2FA is already enabled").extend(),
            );
        }

        let (secret, otpauth_uri) = TotpService::generate_secret(&user.username, "Serve");
        db.set_totp_secret(uid, secret.clone())
            .await
            .map_err(|e| e.extend())?;

        Ok(TotpSetupGql {
            secret,
            otpauth_uri,
        })
    }

    /// Enable 2FA after verifying first 6-digit TOTP code
    async fn enable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP setup not initiated").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        db.enable_2fa(uid).await.map_err(|e| e.extend())?;
        Ok(true)
    }

    /// Disable 2FA by verifying 6-digit TOTP code
    async fn disable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let user = db
            .get_user_by_id(uid)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::UserNotFound, "User not found").extend())?;

        if !user.is_2fa_enabled {
            return Err(DomainError::new(ErrorCode::TotpNotEnabled, "2FA is not enabled").extend());
        }

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP secret missing").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        db.disable_2fa(uid).await.map_err(|e| e.extend())?;
        Ok(true)
    }
}
