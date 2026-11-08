use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::application::totp_service::TotpService;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::graphql::types::{AuthPayloadGql, TotpSetupGql, UserGql};
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
        let services = ctx.data::<AppServices>()?;
        let (pair, user) = services
            .auth
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
            token: pair.access_token,
            refresh_token: Some(pair.refresh_token),
            expires_in: Some(pair.access_expires_in),
            token_type: Some(pair.token_type),
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
        let services = ctx.data::<AppServices>()?;
        let (pair, user) = services
            .auth
            .login(&username_or_email, &password)
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token: pair.access_token,
            refresh_token: Some(pair.refresh_token),
            expires_in: Some(pair.access_expires_in),
            token_type: Some(pair.token_type),
            user: UserGql(user),
        })
    }

    /// Refresh access token using a valid refresh token (Refresh Token Rotation - RTR)
    async fn refresh_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> Result<AuthPayloadGql> {
        let services = ctx.data::<AppServices>()?;
        let (pair, user) = services
            .auth
            .refresh_tokens(&refresh_token)
            .await
            .map_err(|e| e.extend())?;

        Ok(AuthPayloadGql {
            token: pair.access_token,
            refresh_token: Some(pair.refresh_token),
            expires_in: Some(pair.access_expires_in),
            token_type: Some(pair.token_type),
            user: UserGql(user),
        })
    }

    /// Logout and revoke refresh token session
    async fn logout(&self, ctx: &Context<'_>, refresh_token: Option<String>) -> Result<bool> {
        let auth_user = ctx.data_opt::<crate::infrastructure::auth::AuthUser>();
        let services = ctx.data::<AppServices>()?;

        if let Some(user) = auth_user {
            services
                .auth
                .logout(user.user_id, refresh_token.as_deref())
                .await
                .map_err(|e| e.extend())?;
            Ok(true)
        } else if let Some(ref token) = refresh_token {
            let (ref_user, _, _) = crate::infrastructure::auth::verify_refresh_token(
                token,
                &ctx.data::<crate::infrastructure::config::AuthConfig>()?
                    .jwt_secret,
            )
            .map_err(|e| e.extend())?;
            services
                .auth
                .logout(ref_user.user_id, Some(token))
                .await
                .map_err(|e| e.extend())?;
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::AuthTokenRequired,
                "Authentication required to logout",
            )
            .extend())
        }
    }

    /// Update user profile details
    async fn update_user_profile(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        username: Option<String>,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<UserGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let user = services
            .social
            .update_user_profile(
                uid,
                username,
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
        let services = ctx.data::<AppServices>()?;
        let user = services
            .social
            .update_user_privacy(uid, is_private)
            .await
            .map_err(|e| e.extend())?;
        Ok(UserGql(user))
    }

    /// Generate TOTP secret and QR/auth URI for 2FA setup
    #[graphql(name = "setup2fa")]
    async fn setup_2fa(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<TotpSetupGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let user =
            services.social.get_user_by_id(uid).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;

        if user.is_2fa_enabled {
            return Err(
                DomainError::new(ErrorCode::TotpAlreadyEnabled, "2FA is already enabled").extend(),
            );
        }

        let (secret, otpauth_uri) = TotpService::generate_secret(&user.username, "Serve");
        services
            .social
            .set_totp_secret(uid, secret.clone())
            .await
            .map_err(|e| e.extend())?;

        Ok(TotpSetupGql {
            secret,
            otpauth_uri,
        })
    }

    /// Enable 2FA after verifying first 6-digit TOTP code
    #[graphql(name = "enable2fa")]
    async fn enable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let user =
            services.social.get_user_by_id(uid).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP setup not initiated").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        services
            .social
            .enable_2fa(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(true)
    }

    /// Disable 2FA by verifying 6-digit TOTP code
    #[graphql(name = "disable2fa")]
    async fn disable_2fa(
        &self,
        ctx: &Context<'_>,
        code: String,
        user_id: Option<ID>,
    ) -> Result<bool> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let user =
            services.social.get_user_by_id(uid).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;

        if !user.is_2fa_enabled {
            return Err(DomainError::new(ErrorCode::TotpNotEnabled, "2FA is not enabled").extend());
        }

        let secret = user.totp_secret.ok_or_else(|| {
            DomainError::new(ErrorCode::TotpNotEnabled, "TOTP secret missing").extend()
        })?;

        TotpService::verify_code(&secret, &code).map_err(|e| e.extend())?;
        services
            .social
            .disable_2fa(uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(true)
    }
}
