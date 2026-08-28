use async_graphql::{ErrorExtensions, Value};
use serve::domain::errors::{DomainError, ErrorCode};

#[test]
fn test_error_code_string_representation() {
    assert_eq!(
        ErrorCode::AuthInvalidCredentials.as_str(),
        "AUTH_INVALID_CREDENTIALS"
    );
    assert_eq!(ErrorCode::UserNotFound.as_str(), "USER_NOT_FOUND");
    assert_eq!(
        ErrorCode::UserCannotFollowSelf.as_str(),
        "USER_CANNOT_FOLLOW_SELF"
    );
    assert_eq!(ErrorCode::StoryNotFound.as_str(), "STORY_NOT_FOUND");
    assert_eq!(ErrorCode::PostNotFound.as_str(), "POST_NOT_FOUND");
    assert_eq!(
        ErrorCode::DmCannotSendToSelf.as_str(),
        "DM_CANNOT_SEND_TO_SELF"
    );
    assert_eq!(
        ErrorCode::ParentCommentNotFound.as_str(),
        "PARENT_COMMENT_NOT_FOUND"
    );
    assert_eq!(
        ErrorCode::NotificationNotFound.as_str(),
        "NOTIFICATION_NOT_FOUND"
    );
    assert_eq!(ErrorCode::InvalidCursor.as_str(), "INVALID_CURSOR");
    assert_eq!(ErrorCode::UserBlocked.as_str(), "USER_BLOCKED");
    assert_eq!(ErrorCode::CannotBlockSelf.as_str(), "CANNOT_BLOCK_SELF");
    assert_eq!(
        ErrorCode::FollowRequestNotFound.as_str(),
        "FOLLOW_REQUEST_NOT_FOUND"
    );
    assert_eq!(
        ErrorCode::PostAlreadyReposted.as_str(),
        "POST_ALREADY_REPOSTED"
    );
    assert_eq!(
        ErrorCode::ConversationNotFound.as_str(),
        "CONVERSATION_NOT_FOUND"
    );
}

#[test]
fn test_domain_error_with_params() {
    let err = DomainError::with_params(
        ErrorCode::AuthUserAlreadyExists,
        "A user with this username or email already exists",
        [("username", "ferro_dev"), ("email", "ferro@example.com")],
    );

    assert_eq!(err.code, ErrorCode::AuthUserAlreadyExists);
    assert_eq!(err.params.get("username").unwrap(), "ferro_dev");
    assert_eq!(err.params.get("email").unwrap(), "ferro@example.com");

    let gql_err = err.extend();
    assert_eq!(gql_err.message, "AUTH_USER_ALREADY_EXISTS");
    let ext = gql_err.extensions.expect("Expected GraphQL extensions");
    assert_eq!(
        ext.get("code"),
        Some(&Value::String("AUTH_USER_ALREADY_EXISTS".to_string()))
    );

    if let Some(Value::Object(map)) = ext.get("params") {
        assert_eq!(
            map.get("username"),
            Some(&Value::String("ferro_dev".to_string()))
        );
        assert_eq!(
            map.get("email"),
            Some(&Value::String("ferro@example.com".to_string()))
        );
    } else {
        panic!("Expected Object for params extension");
    }
}

#[test]
fn test_domain_error_builder_with_param() {
    let err = DomainError::new(ErrorCode::StoryCreateFailed, "Failed to create story")
        .with_param("detail", "Image size exceeds 10MB");

    assert_eq!(err.code, ErrorCode::StoryCreateFailed);
    assert_eq!(err.params.get("detail").unwrap(), "Image size exceeds 10MB");

    let gql_err = err.extend();
    let ext = gql_err.extensions.unwrap();
    assert_eq!(
        ext.get("code"),
        Some(&Value::String("STORY_CREATE_FAILED".to_string()))
    );

    if let Some(Value::Object(map)) = ext.get("params") {
        assert_eq!(
            map.get("detail"),
            Some(&Value::String("Image size exceeds 10MB".to_string()))
        );
    } else {
        panic!("Expected Object for params extension");
    }
}
