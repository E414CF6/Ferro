use crate::domain::errors::{DomainError, ErrorCode};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct TotpService;

impl TotpService {
    /// Generate a new base32 secret and otpauth URI
    pub fn generate_secret(username: &str, issuer: &str) -> (String, String) {
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes().unwrap_or_default(),
            Some(issuer.to_string()),
            username.to_string(),
        )
        .unwrap();

        let uri = totp.get_url();
        (secret_base32, uri)
    }

    /// Verify a 6-digit TOTP code
    pub fn verify_code(secret_base32: &str, code: &str) -> Result<bool, DomainError> {
        let secret = Secret::Encoded(secret_base32.to_string());
        let secret_bytes = secret.to_bytes().map_err(|_| {
            DomainError::new(ErrorCode::TotpInvalidCode, "Invalid TOTP secret format")
        })?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            None,
            "".to_string(),
        )
        .map_err(|_| DomainError::new(ErrorCode::TotpInvalidCode, "Failed to initialize TOTP"))?;

        let valid = totp.check_current(code).unwrap_or(false);
        if valid {
            Ok(true)
        } else {
            Err(DomainError::new(
                ErrorCode::TotpInvalidCode,
                "Invalid 6-digit authentication code",
            ))
        }
    }
}
