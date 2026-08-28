use crate::domain::errors::{DomainError, ErrorCode};

const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "administrator",
    "root",
    "api",
    "graphql",
    "graphiql",
    "me",
    "system",
    "ferro",
    "null",
    "undefined",
    "bot",
    "support",
    "help",
];

/// Validates user registration inputs
pub fn validate_username(username: &str) -> Result<(), DomainError> {
    let trimmed = username.trim();
    if trimmed.len() < 3 || trimmed.len() > 50 {
        return Err(DomainError::new(
            ErrorCode::AuthInvalidUsername,
            ErrorCode::AuthInvalidUsername.as_str(),
        ));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(DomainError::new(
            ErrorCode::AuthInvalidUsername,
            ErrorCode::AuthInvalidUsername.as_str(),
        ));
    }

    if RESERVED_USERNAMES.contains(&trimmed.to_lowercase().as_str()) {
        return Err(DomainError::new(
            ErrorCode::AuthInvalidUsername,
            ErrorCode::AuthInvalidUsername.as_str(),
        ));
    }

    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), DomainError> {
    let trimmed = email.trim();
    if trimmed.len() < 5 || trimmed.len() > 254 || !trimmed.contains('@') {
        return Err(DomainError::new(
            ErrorCode::AuthInvalidEmail,
            ErrorCode::AuthInvalidEmail.as_str(),
        ));
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2
        || parts[0].is_empty()
        || !parts[1].contains('.')
        || parts[1].starts_with('.')
        || parts[1].ends_with('.')
    {
        return Err(DomainError::new(
            ErrorCode::AuthInvalidEmail,
            ErrorCode::AuthInvalidEmail.as_str(),
        ));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), DomainError> {
    if password.len() < 8 {
        return Err(DomainError::new(
            ErrorCode::AuthPasswordTooShort,
            ErrorCode::AuthPasswordTooShort.as_str(),
        ));
    }
    if password.len() > 128 {
        return Err(DomainError::new(
            ErrorCode::AuthPasswordTooShort,
            ErrorCode::AuthPasswordTooShort.as_str(),
        ));
    }
    Ok(())
}

pub fn validate_post_content(content: &str) -> Result<String, DomainError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(DomainError::new(
            ErrorCode::PostContentInvalid,
            ErrorCode::PostContentInvalid.as_str(),
        ));
    }
    if trimmed.chars().count() > 2000 {
        return Err(DomainError::new(
            ErrorCode::PostContentInvalid,
            ErrorCode::PostContentInvalid.as_str(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn validate_comment_content(content: &str) -> Result<String, DomainError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(DomainError::new(
            ErrorCode::CommentContentInvalid,
            ErrorCode::CommentContentInvalid.as_str(),
        ));
    }
    if trimmed.chars().count() > 500 {
        return Err(DomainError::new(
            ErrorCode::CommentContentInvalid,
            ErrorCode::CommentContentInvalid.as_str(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn validate_dm_content(content: &str) -> Result<String, DomainError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(DomainError::new(
            ErrorCode::DmContentInvalid,
            ErrorCode::DmContentInvalid.as_str(),
        ));
    }
    if trimmed.chars().count() > 2000 {
        return Err(DomainError::new(
            ErrorCode::DmContentInvalid,
            ErrorCode::DmContentInvalid.as_str(),
        ));
    }
    Ok(trimmed.to_string())
}
