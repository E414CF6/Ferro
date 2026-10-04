use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serve::application::{
    auth_service::AuthService, comment_service::CommentService, post_service::PostService,
    wiki_service::WikiService,
};
use serve::domain::errors::{DomainError, ErrorCode};
use serve::domain::models::{
    ArticleFilterParams, Comment, FollowRequest, Post, PostAnalytics, PostAudience,
    PostMedia, User, UserList, WikiArticle, WikiRevision,
};
use serve::domain::repositories::{
    CommentRepository, PostRepository, UserRepository, WikiRepository,
};
use serve::infrastructure::config::AuthConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// ============================================================================
// 1. MockCommentRepo & CommentService Tests
// ============================================================================

struct MockCommentRepo;

#[async_trait]
impl CommentRepository for MockCommentRepo {
    async fn create_comment(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
    ) -> Result<Comment, DomainError> {
        Ok(Comment {
            id: Uuid::new_v4(),
            post_id,
            author_id,
            content,
            parent_id,
            is_edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn edit_comment(
        &self,
        comment_id: Uuid,
        author_id: Uuid,
        new_content: String,
    ) -> Result<Comment, DomainError> {
        Ok(Comment {
            id: comment_id,
            post_id: Uuid::new_v4(),
            author_id,
            content: new_content,
            parent_id: None,
            is_edited: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn delete_comment(
        &self,
        _comment_id: Uuid,
        _author_id: Uuid,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn get_comment_by_id(&self, comment_id: Uuid) -> Option<Comment> {
        Some(Comment {
            id: comment_id,
            post_id: Uuid::new_v4(),
            author_id: Uuid::new_v4(),
            content: "Existing comment".to_string(),
            parent_id: None,
            is_edited: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_comments_for_post(&self, _post_id: Uuid) -> Vec<Comment> {
        vec![]
    }

    async fn get_top_level_comments_for_post(&self, _post_id: Uuid) -> Vec<Comment> {
        vec![]
    }

    async fn get_comments_cursor(
        &self,
        _post_id: Uuid,
        _top_level_only: bool,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        (vec![], false)
    }

    async fn get_replies_for_comment(&self, _comment_id: Uuid) -> Vec<Comment> {
        vec![]
    }

    async fn get_replies_cursor(
        &self,
        _comment_id: Uuid,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Comment>, bool) {
        (vec![], false)
    }

    async fn get_replies_count(&self, _comment_id: Uuid) -> usize {
        2
    }

    async fn like_comment(&self, _user_id: Uuid, comment_id: Uuid) -> Result<Comment, DomainError> {
        let comment = self
            .get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))?;
        Ok(comment)
    }

    async fn unlike_comment(
        &self,
        _user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Comment, DomainError> {
        let comment = self
            .get_comment_by_id(comment_id)
            .await
            .ok_or_else(|| DomainError::new(ErrorCode::CommentNotFound, "Comment not found"))?;
        Ok(comment)
    }

    async fn get_comment_likes_count(&self, _comment_id: Uuid) -> usize {
        42
    }

    async fn is_comment_liked_by(&self, _comment_id: Uuid, _user_id: Uuid) -> bool {
        true
    }

    async fn pin_comment(
        &self,
        _post_id: Uuid,
        _comment_id: Uuid,
        _author_id: Uuid,
    ) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn unpin_comment(&self, _post_id: Uuid, _author_id: Uuid) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn get_pinned_comment(&self, _post_id: Uuid) -> Option<Comment> {
        None
    }
}

#[tokio::test]
async fn test_comment_service_with_mock_repository_creation() {
    let mock_repo = Arc::new(MockCommentRepo);
    let service = CommentService::new(mock_repo);

    let post_id = Uuid::new_v4();
    let author_id = Uuid::new_v4();

    // 1. Valid comment creation
    let result = service
        .create_comment(
            post_id,
            author_id,
            "Hello Ferro architecture!".to_string(),
            None,
        )
        .await;
    assert!(result.is_ok());
    let comment = result.unwrap();
    assert_eq!(comment.content, "Hello Ferro architecture!");
    assert!(!comment.is_edited);

    // 2. Validation failure: empty content
    let empty_result = service
        .create_comment(post_id, author_id, "   ".to_string(), None)
        .await;
    assert!(empty_result.is_err());
    let err = empty_result.unwrap_err();
    assert_eq!(err.code, ErrorCode::CommentContentInvalid);
}

#[tokio::test]
async fn test_comment_service_edit_validation() {
    let mock_repo = Arc::new(MockCommentRepo);
    let service = CommentService::new(mock_repo);

    let comment_id = Uuid::new_v4();
    let author_id = Uuid::new_v4();

    let edit_res = service
        .edit_comment(comment_id, author_id, "Updated content".to_string())
        .await;
    assert!(edit_res.is_ok());
    assert!(edit_res.unwrap().is_edited);
}

// ============================================================================
// 2. MockPostRepo & PostService Tests
// ============================================================================

#[derive(Default)]
struct MockPostRepo {
    posts: Mutex<Vec<Post>>,
}

#[async_trait]
impl PostRepository for MockPostRepo {
    async fn get_user_posts_count(&self, user_id: Uuid) -> usize {
        let lock = self.posts.lock().await;
        lock.iter().filter(|p| p.author_id == user_id).count()
    }

    async fn get_liked_posts_by_user(&self, _user_id: Uuid) -> Vec<Post> {
        vec![]
    }

    async fn get_comments_by_user(&self, _user_id: Uuid) -> Vec<Comment> {
        vec![]
    }

    async fn get_posts(&self, _limit: Option<usize>, _offset: Option<usize>) -> Vec<Post> {
        self.posts.lock().await.clone()
    }

    async fn get_posts_cursor(
        &self,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        (self.posts.lock().await.clone(), false)
    }

    async fn get_post_by_id(&self, id: Uuid) -> Option<Post> {
        let lock = self.posts.lock().await;
        lock.iter().find(|p| p.id == id).cloned()
    }

    async fn get_posts_by_author(&self, author_id: Uuid) -> Vec<Post> {
        let lock = self.posts.lock().await;
        lock.iter().filter(|p| p.author_id == author_id).cloned().collect()
    }

    async fn create_post(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
    ) -> Result<Post, DomainError> {
        let post = Post {
            id: Uuid::new_v4(),
            author_id,
            content,
            audience: audience.unwrap_or(PostAudience::Public),
            views_count: 0,
            quote_post_id: None,
            pinned_comment_id: None,
            created_at: Utc::now(),
        };
        self.posts.lock().await.push(post.clone());
        Ok(post)
    }

    async fn create_post_with_details(
        &self,
        author_id: Uuid,
        content: String,
        audience: Option<PostAudience>,
        _media: Vec<PostMedia>,
        _poll: Option<(String, Vec<String>, i64)>,
    ) -> Result<Post, DomainError> {
        self.create_post(author_id, content, audience).await
    }

    async fn create_quote_post(
        &self,
        author_id: Uuid,
        quote_post_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        let post = Post {
            id: Uuid::new_v4(),
            author_id,
            content,
            audience: PostAudience::Public,
            views_count: 0,
            quote_post_id: Some(quote_post_id),
            pinned_comment_id: None,
            created_at: Utc::now(),
        };
        self.posts.lock().await.push(post.clone());
        Ok(post)
    }

    async fn update_post(
        &self,
        post_id: Uuid,
        author_id: Uuid,
        content: String,
    ) -> Result<Post, DomainError> {
        let mut lock = self.posts.lock().await;
        let post = lock.iter_mut().find(|p| p.id == post_id).ok_or_else(|| {
            DomainError::new(ErrorCode::PostNotFound, "Post not found")
        })?;

        if post.author_id != author_id {
            return Err(DomainError::new(
                ErrorCode::ErrorForbidden,
                "Unauthorized: Not post author",
            ));
        }

        post.content = content;
        Ok(post.clone())
    }

    async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<bool, DomainError> {
        let mut lock = self.posts.lock().await;
        if let Some(pos) = lock.iter().position(|p| p.id == post_id) {
            if lock[pos].author_id != author_id {
                return Err(DomainError::new(
                    ErrorCode::ErrorForbidden,
                    "Unauthorized to delete",
                ));
            }
            lock.remove(pos);
            Ok(true)
        } else {
            Err(DomainError::new(ErrorCode::PostNotFound, "Post not found"))
        }
    }

    async fn get_feed(
        &self,
        _user_id: Uuid,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> Vec<Post> {
        self.posts.lock().await.clone()
    }

    async fn get_feed_cursor(
        &self,
        _user_id: Uuid,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        (self.posts.lock().await.clone(), false)
    }

    async fn search_posts(&self, query: &str) -> Vec<Post> {
        let lock = self.posts.lock().await;
        lock.iter().filter(|p| p.content.contains(query)).cloned().collect()
    }

    async fn search_posts_cursor(
        &self,
        _query: &str,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        (vec![], false)
    }

    async fn get_posts_by_hashtag_cursor(
        &self,
        _hashtag: &str,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        (vec![], false)
    }

    async fn get_trending_hashtags(&self, _limit: Option<usize>) -> Vec<(String, usize)> {
        vec![]
    }

    async fn like_post(&self, _user_id: Uuid, _post_id: Uuid) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn unlike_post(&self, _user_id: Uuid, _post_id: Uuid) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn is_post_liked_by(&self, _post_id: Uuid, _user_id: Uuid) -> bool {
        false
    }

    async fn get_likes_count(&self, _post_id: Uuid) -> usize {
        0
    }

    async fn repost_post(&self, _user_id: Uuid, _post_id: Uuid) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn unrepost_post(&self, _user_id: Uuid, _post_id: Uuid) -> Result<Post, DomainError> {
        unimplemented!()
    }

    async fn is_post_reposted_by(&self, _post_id: Uuid, _user_id: Uuid) -> bool {
        false
    }

    async fn get_reposts_count(&self, _post_id: Uuid) -> usize {
        0
    }

    async fn add_post_media(
        &self,
        _post_id: Uuid,
        _media: Vec<PostMedia>,
    ) -> Result<Vec<PostMedia>, DomainError> {
        Ok(vec![])
    }

    async fn get_post_media(&self, _post_id: Uuid) -> Vec<PostMedia> {
        vec![]
    }

    async fn create_user_list(
        &self,
        _owner_id: Uuid,
        _name: String,
        _description: Option<String>,
        _is_private: bool,
        _member_ids: Vec<Uuid>,
    ) -> Result<UserList, DomainError> {
        unimplemented!()
    }

    async fn update_user_list(
        &self,
        _list_id: Uuid,
        _owner_id: Uuid,
        _name: Option<String>,
        _description: Option<String>,
        _is_private: Option<bool>,
    ) -> Result<UserList, DomainError> {
        unimplemented!()
    }

    async fn delete_user_list(&self, _list_id: Uuid, _owner_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn add_user_to_list(
        &self,
        _list_id: Uuid,
        _owner_id: Uuid,
        _user_id: Uuid,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn remove_user_from_list(
        &self,
        _list_id: Uuid,
        _owner_id: Uuid,
        _user_id: Uuid,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn get_user_lists(&self, _user_id: Uuid) -> Vec<UserList> {
        vec![]
    }

    async fn get_user_list_by_id(&self, _list_id: Uuid) -> Option<UserList> {
        None
    }

    async fn get_list_members(&self, _list_id: Uuid) -> Vec<User> {
        vec![]
    }

    async fn get_list_feed_cursor(
        &self,
        _list_id: Uuid,
        _first: usize,
        _after: Option<(DateTime<Utc>, Uuid)>,
    ) -> (Vec<Post>, bool) {
        (vec![], false)
    }

    async fn record_post_view(&self, _post_id: Uuid, _viewer_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }

    async fn get_post_analytics(&self, post_id: Uuid) -> Result<PostAnalytics, DomainError> {
        Ok(PostAnalytics {
            post_id,
            views_count: 100,
            likes_count: 10,
            reposts_count: 5,
            comments_count: 2,
            engagement_rate: 0.17,
        })
    }
}

#[tokio::test]
async fn test_post_service_create_and_validation() {
    let mock_repo = Arc::new(MockPostRepo::default());
    let service = PostService::new(mock_repo);
    let author_id = Uuid::new_v4();

    // 1. Success case
    let post = service
        .create_post(
            author_id,
            "Valid post with #hashtag and @mention".to_string(),
            Some(PostAudience::Public),
        )
        .await
        .expect("Post creation should succeed");
    assert_eq!(post.author_id, author_id);
    assert_eq!(post.content, "Valid post with #hashtag and @mention");

    // 2. Validation: Empty content
    let empty_err = service
        .create_post(author_id, "     ".to_string(), None)
        .await;
    assert!(empty_err.is_err());
    assert_eq!(empty_err.unwrap_err().code, ErrorCode::PostContentInvalid);

    // 3. Validation: Exceeding character limit (max 2000 chars)
    let too_long_content = "A".repeat(2001);
    let long_err = service.create_post(author_id, too_long_content, None).await;
    assert!(long_err.is_err());
    assert_eq!(long_err.unwrap_err().code, ErrorCode::PostContentInvalid);
}

#[tokio::test]
async fn test_post_service_quote_and_update_permissions() {
    let mock_repo = Arc::new(MockPostRepo::default());
    let service = PostService::new(mock_repo);
    let author_id = Uuid::new_v4();
    let original_post_id = Uuid::new_v4();

    // 1. Quote Post
    let quote = service
        .create_quote_post(
            author_id,
            original_post_id,
            "Quoting an inspiring post".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(quote.quote_post_id, Some(original_post_id));

    // 2. Author can update post
    let updated = service
        .update_post(quote.id, author_id, "Updated quote content".to_string())
        .await
        .unwrap();
    assert_eq!(updated.content, "Updated quote content");

    // 3. Non-author cannot update post
    let unauthorized_id = Uuid::new_v4();
    let unauthorized_err = service
        .update_post(quote.id, unauthorized_id, "Malicious edit".to_string())
        .await;
    assert!(unauthorized_err.is_err());
    assert_eq!(
        unauthorized_err.unwrap_err().code,
        ErrorCode::ErrorForbidden
    );
}

// ============================================================================
// 3. MockUserRepo & AuthService Tests
// ============================================================================

#[derive(Default)]
struct MockUserRepo {
    users: Mutex<Vec<User>>,
}

#[async_trait]
impl UserRepository for MockUserRepo {
    async fn get_users(&self) -> Vec<User> {
        self.users.lock().await.clone()
    }

    async fn get_user_by_id(&self, id: Uuid) -> Option<User> {
        let lock = self.users.lock().await;
        lock.iter().find(|u| u.id == id).cloned()
    }

    async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let lock = self.users.lock().await;
        lock.iter().find(|u| u.username == username).cloned()
    }

    async fn get_user_by_identifier(&self, identifier: &str) -> Option<User> {
        let lock = self.users.lock().await;
        lock.iter()
            .find(|u| u.username == identifier || u.email == identifier)
            .cloned()
    }

    async fn register_user(
        &self,
        username: String,
        email: String,
        password_hash: String,
        display_name: String,
        bio: Option<String>,
        avatar_url: Option<String>,
        header_image_url: Option<String>,
        location: Option<String>,
        website: Option<String>,
    ) -> Result<User, DomainError> {
        let mut lock = self.users.lock().await;
        if lock.iter().any(|u| u.username == username || u.email == email) {
            return Err(DomainError::new(
                ErrorCode::AuthUserAlreadyExists,
                "User already exists",
            ));
        }

        let user = User {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            display_name,
            bio,
            avatar_url,
            header_image_url,
            location,
            website,
            is_private: false,
            is_2fa_enabled: false,
            totp_secret: None,
            created_at: Utc::now(),
        };
        lock.push(user.clone());
        Ok(user)
    }

    async fn update_user_profile(
        &self,
        _user_id: Uuid,
        _display_name: Option<String>,
        _bio: Option<String>,
        _avatar_url: Option<String>,
        _header_image_url: Option<String>,
        _location: Option<String>,
        _website: Option<String>,
    ) -> Result<User, DomainError> {
        unimplemented!()
    }

    async fn update_user_privacy(
        &self,
        _user_id: Uuid,
        _is_private: bool,
    ) -> Result<User, DomainError> {
        unimplemented!()
    }

    async fn is_following(&self, _follower_id: Uuid, _followee_id: Uuid) -> bool {
        false
    }
    async fn follow_user(&self, _follower_id: Uuid, _followee_id: Uuid) -> Result<User, DomainError> {
        unimplemented!()
    }
    async fn unfollow_user(
        &self,
        _follower_id: Uuid,
        _followee_id: Uuid,
    ) -> Result<User, DomainError> {
        unimplemented!()
    }
    async fn get_followers(&self, _user_id: Uuid) -> Vec<User> {
        vec![]
    }
    async fn get_following(&self, _user_id: Uuid) -> Vec<User> {
        vec![]
    }
    async fn get_followers_count(&self, _user_id: Uuid) -> usize {
        0
    }
    async fn get_following_count(&self, _user_id: Uuid) -> usize {
        0
    }
    async fn search_users(&self, _query: &str, _limit: Option<usize>) -> Vec<User> {
        vec![]
    }
    async fn block_user(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn unblock_user(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn is_blocking(&self, _blocker_id: Uuid, _blocked_id: Uuid) -> bool {
        false
    }
    async fn is_blocked_between(&self, _user_a: Uuid, _user_b: Uuid) -> bool {
        false
    }
    async fn mute_user(&self, _muter_id: Uuid, _muted_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn unmute_user(&self, _muter_id: Uuid, _muted_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn is_muting(&self, _muter_id: Uuid, _muted_id: Uuid) -> bool {
        false
    }
    async fn enable_2fa(&self, _user_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn disable_2fa(&self, _user_id: Uuid) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn create_follow_request(
        &self,
        _req_id: Uuid,
        _tgt_id: Uuid,
    ) -> Result<FollowRequest, DomainError> {
        unimplemented!()
    }
    async fn accept_follow_request(
        &self,
        _tgt_id: Uuid,
        _req_id: Uuid,
    ) -> Result<User, DomainError> {
        unimplemented!()
    }
    async fn reject_follow_request(
        &self,
        _tgt_id: Uuid,
        _req_id: Uuid,
    ) -> Result<bool, DomainError> {
        Ok(true)
    }
    async fn get_pending_follow_requests(&self, _tgt_id: Uuid) -> Vec<FollowRequest> {
        vec![]
    }
    async fn has_pending_follow_request(&self, _req_id: Uuid, _tgt_id: Uuid) -> bool {
        false
    }
    async fn get_pending_follow_requests_count(&self, _tgt_id: Uuid) -> usize {
        0
    }
    async fn set_totp_secret(&self, _user_id: Uuid, _secret: String) -> Result<bool, DomainError> {
        Ok(true)
    }
}

#[tokio::test]
async fn test_auth_service_signup_and_login_flow() {
    let mock_repo = Arc::new(MockUserRepo::default());
    let auth_config = AuthConfig {
        jwt_secret: "super-secure-secret-key-for-unit-testing".to_string(),
        jwt_expiration_days: 7,
    };
    let service = AuthService::new(mock_repo, auth_config);

    // 1. Successful Signup
    let (token, user) = service
        .signup(
            "alice_dip".to_string(),
            "alice@example.com".to_string(),
            "StrongPassword123!".to_string(),
            "Alice DIP".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Signup should succeed");
    assert!(!token.is_empty());
    assert_eq!(user.username, "alice_dip");

    // 2. Login by Username
    let (login_token, login_user) = service
        .login("alice_dip", "StrongPassword123!")
        .await
        .expect("Login with username should succeed");
    assert!(!login_token.is_empty());
    assert_eq!(login_user.id, user.id);

    // 3. Login by Email
    let (email_token, email_user) = service
        .login("alice@example.com", "StrongPassword123!")
        .await
        .expect("Login with email should succeed");
    assert!(!email_token.is_empty());
    assert_eq!(email_user.id, user.id);

    // 4. Login with Wrong Password
    let wrong_pw_err = service.login("alice_dip", "WrongPassword!").await;
    assert!(wrong_pw_err.is_err());
    assert_eq!(
        wrong_pw_err.unwrap_err().code,
        ErrorCode::AuthInvalidCredentials
    );

    // 5. Login with Nonexistent User
    let not_found_err = service.login("ghost_user", "AnyPassword!").await;
    assert!(not_found_err.is_err());
    assert_eq!(
        not_found_err.unwrap_err().code,
        ErrorCode::AuthInvalidCredentials
    );
}

#[tokio::test]
async fn test_auth_service_signup_validation_failures() {
    let mock_repo = Arc::new(MockUserRepo::default());
    let auth_config = AuthConfig::default();
    let service = AuthService::new(mock_repo, auth_config);

    // 1. Password too short (< 8 chars)
    let short_pw = service
        .signup(
            "valid_user".to_string(),
            "valid@example.com".to_string(),
            "short".to_string(),
            "Valid User".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(short_pw.is_err());
    assert_eq!(short_pw.unwrap_err().code, ErrorCode::AuthPasswordTooShort);

    // 2. Invalid Username characters (contains forbidden exclamation mark)
    let bad_uname = service
        .signup(
            "invalid!username".to_string(),
            "valid@example.com".to_string(),
            "StrongPassword123!".to_string(),
            "Valid User".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(bad_uname.is_err());
    assert_eq!(bad_uname.unwrap_err().code, ErrorCode::AuthInvalidUsername);

    // 3. Invalid Email Format
    let bad_email = service
        .signup(
            "valid_uname".to_string(),
            "not-an-email".to_string(),
            "StrongPassword123!".to_string(),
            "Valid User".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(bad_email.is_err());
    assert_eq!(bad_email.unwrap_err().code, ErrorCode::AuthInvalidEmail);
}

// ============================================================================
// 4. MockWikiRepo & WikiService Tests
// ============================================================================

#[derive(Default)]
struct MockWikiRepo {
    articles: Mutex<Vec<WikiArticle>>,
    revisions: Mutex<Vec<WikiRevision>>,
}

#[async_trait]
impl WikiRepository for MockWikiRepo {
    async fn find_articles(
        &self,
        _filters: &ArticleFilterParams,
    ) -> Result<Vec<WikiArticle>, DomainError> {
        Ok(self.articles.lock().await.clone())
    }

    async fn find_article_by_slug(&self, slug: &str) -> Result<Option<WikiArticle>, DomainError> {
        let lock = self.articles.lock().await;
        Ok(lock.iter().find(|a| a.slug == slug).cloned())
    }

    async fn find_article_by_id(&self, id: Uuid) -> Result<Option<WikiArticle>, DomainError> {
        let lock = self.articles.lock().await;
        Ok(lock.iter().find(|a| a.id == id).cloned())
    }

    async fn find_slugs_by_prefix(&self, base_slug: &str) -> Result<Vec<String>, DomainError> {
        let lock = self.articles.lock().await;
        let prefix = format!("{}-", base_slug);
        let slugs = lock
            .iter()
            .filter(|a| a.slug == base_slug || a.slug.starts_with(&prefix))
            .map(|a| a.slug.clone())
            .collect();
        Ok(slugs)
    }

    async fn create_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError> {
        self.articles.lock().await.push(article.clone());
        self.revisions.lock().await.push(revision.clone());
        Ok(article.clone())
    }

    async fn update_article(
        &self,
        article: &WikiArticle,
        revision: &WikiRevision,
    ) -> Result<WikiArticle, DomainError> {
        let mut lock = self.articles.lock().await;
        if let Some(pos) = lock.iter().position(|a| a.id == article.id) {
            lock[pos] = article.clone();
            self.revisions.lock().await.push(revision.clone());
            Ok(article.clone())
        } else {
            Err(DomainError::new(ErrorCode::WikiArticleNotFound, "Article not found"))
        }
    }

    async fn delete_article(&self, slug: &str) -> Result<bool, DomainError> {
        let mut lock = self.articles.lock().await;
        if let Some(pos) = lock.iter().position(|a| a.slug == slug) {
            lock.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn record_view(&self, _article_id: Uuid) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_revisions(
        &self,
        article_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<WikiRevision>, DomainError> {
        let lock = self.revisions.lock().await;
        Ok(lock.iter().filter(|r| r.article_id == article_id).cloned().collect())
    }

    async fn get_all_articles_meta(&self) -> Result<Vec<WikiArticle>, DomainError> {
        Ok(self.articles.lock().await.clone())
    }

    async fn get_view_log_counts_since(
        &self,
        _since: DateTime<Utc>,
    ) -> Result<Vec<(Uuid, i64)>, DomainError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_wiki_service_slug_generation_and_uniqueness() {
    let mock_repo = Arc::new(MockWikiRepo::default());
    let service = WikiService::new(mock_repo);

    // 1. Initial slug generation cleans special symbols and spaces
    let slug1 = service
        .generate_unique_slug("Namsan Seoul Tower @ 2026!")
        .await
        .unwrap();
    assert_eq!(slug1, "namsan-seoul-tower-2026");

    // 2. Unicode / Hangul title support
    let slug_ko = service
        .generate_unique_slug("경복궁 근정전 안내")
        .await
        .unwrap();
    assert_eq!(slug_ko, "경복궁-근정전-안내");
}

#[tokio::test]
async fn test_wiki_service_validation_rules() {
    let mock_repo = Arc::new(MockWikiRepo::default());
    let service = WikiService::new(mock_repo);

    // 1. Empty title validation failure
    let empty_title_res = service
        .create_article(
            None,
            None,
            "   ".to_string(),
            "Some content".to_string(),
            None,
            37.5,
            127.0,
            Some(15.0),
            None,
            None,
        )
        .await;
    assert!(empty_title_res.is_err());
    assert_eq!(
        empty_title_res.unwrap_err().code,
        ErrorCode::WikiArticleTitleInvalid
    );

    // 2. Invalid Latitude (> 90.0) validation failure
    let invalid_lat_res = service
        .create_article(
            None,
            None,
            "Valid Title".to_string(),
            "Some content".to_string(),
            None,
            95.0, // Invalid latitude!
            127.0,
            Some(15.0),
            None,
            None,
        )
        .await;
    assert!(invalid_lat_res.is_err());
    assert_eq!(
        invalid_lat_res.unwrap_err().code,
        ErrorCode::WikiArticleCoordinatesInvalid
    );
}
