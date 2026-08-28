use serve::domain::validation::{
    validate_comment_content, validate_dm_content, validate_email, validate_password,
    validate_post_content, validate_username,
};

#[test]
fn test_username_validation() {
    assert!(validate_username("valid_user123").is_ok());
    assert!(validate_username("rust_ace").is_ok());

    // Too short / too long
    assert!(validate_username("ab").is_err());
    assert!(validate_username("a".repeat(51).as_str()).is_err());

    // Invalid characters
    assert!(validate_username("user-name!").is_err());
    assert!(validate_username("user name").is_err());
    assert!(validate_username("user@name").is_err());

    // Reserved names
    assert!(validate_username("admin").is_err());
    assert!(validate_username("root").is_err());
    assert!(validate_username("graphql").is_err());
    assert!(validate_username("ferro").is_err());
}

#[test]
fn test_email_validation() {
    assert!(validate_email("user@example.com").is_ok());
    assert!(validate_email("dev.ferro@sub.domain.co").is_ok());

    assert!(validate_email("notanemail").is_err());
    assert!(validate_email("@missinguser.com").is_err());
    assert!(validate_email("user@nodomain").is_err());
    assert!(validate_email("user@.invalid").is_err());
}

#[test]
fn test_password_validation() {
    assert!(validate_password("securepassword123").is_ok());
    assert!(validate_password("12345678").is_ok());

    // Too short
    assert!(validate_password("1234567").is_err());
}

#[test]
fn test_content_validation() {
    // Post
    assert!(validate_post_content("Hello World!").is_ok());
    assert!(validate_post_content("   ").is_err());
    assert!(validate_post_content("").is_err());
    assert!(validate_post_content(&"a".repeat(2001)).is_err());

    // Comment
    assert!(validate_comment_content("Nice post!").is_ok());
    assert!(validate_comment_content("   ").is_err());
    assert!(validate_comment_content(&"a".repeat(501)).is_err());

    // DM
    assert!(validate_dm_content("Hey there!").is_ok());
    assert!(validate_dm_content("   ").is_err());
    assert!(validate_dm_content(&"a".repeat(2001)).is_err());
}
