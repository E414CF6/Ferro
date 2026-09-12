use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serve::application::comment_service::CommentService;
use serve::domain::errors::{DomainError, ErrorCode};
use serve::domain::models::{Comment, Post};
use serve::domain::repositories::CommentRepository;
use std::sync::Arc;
use uuid::Uuid;

/// In-memory mock implementing CommentRepository without requiring a SQLite or Postgres database
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
        let comment = self.get_comment_by_id(comment_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::CommentNotFound, "Comment not found")
        })?;
        Ok(comment)
    }

    async fn unlike_comment(
        &self,
        _user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<Comment, DomainError> {
        let comment = self.get_comment_by_id(comment_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::CommentNotFound, "Comment not found")
        })?;
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
        .create_comment(post_id, author_id, "Hello Ferro architecture!".to_string(), None)
        .await;
    assert!(result.is_ok());
    let comment = result.unwrap();
    assert_eq!(comment.content, "Hello Ferro architecture!");
    assert_eq!(comment.is_edited, false);

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
