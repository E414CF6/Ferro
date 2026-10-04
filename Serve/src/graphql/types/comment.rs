use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::Comment as CommentModel;
use crate::domain::repositories::*;
use crate::infrastructure::auth::AuthUser;
use crate::infrastructure::db::loaders::{
    CommentLikesCountLoader, CommentLoader, CommentRepliesCountLoader, PostLoader, UserLoader,
};
use crate::infrastructure::db::postgres::Database;

use super::{decode_cursor, encode_cursor, PageInfoGql, PostGql, UserGql};

#[derive(Clone)]
pub struct CommentGql(pub CommentModel);

#[Object]
impl CommentGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn parent_id(&self) -> Option<ID> {
        self.0.parent_id.map(|id| ID(id.to_string()))
    }

    async fn content(&self) -> &str {
        &self.0.content
    }

    async fn is_edited(&self) -> bool {
        self.0.is_edited
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        self.0.updated_at
    }

    /// Comment Author (batch loaded via DataLoader)
    async fn author(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.author_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::CommentAuthorNotFound, "Comment author not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.author_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::CommentAuthorNotFound, "Comment author not found")
                    .extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn post(&self, ctx: &Context<'_>) -> Result<PostGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<PostLoader>>() {
            let post = loader.load_one(self.0.post_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::PostNotFound, "Post not found").extend()
            })?;
            Ok(PostGql(post))
        } else {
            let db = ctx.data::<Database>()?;
            let post = db.get_post_by_id(self.0.post_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::PostNotFound, "Post not found").extend()
            })?;
            Ok(PostGql(post))
        }
    }

    async fn parent(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(pid) = self.0.parent_id else {
            return Ok(None);
        };
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLoader>>() {
            let comment_opt = loader.load_one(pid).await?;
            Ok(comment_opt.map(CommentGql))
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_by_id(pid).await.map(CommentGql))
        }
    }

    /// Depth in comment thread hierarchy (0 = root top-level comment)
    async fn depth(&self, ctx: &Context<'_>) -> Result<usize> {
        let mut curr_parent = self.0.parent_id;
        let mut depth = 0;
        let db = ctx.data::<Database>()?;
        while let Some(pid) = curr_parent {
            depth += 1;
            if depth > 20 {
                break;
            }
            if let Some(parent) = db.get_comment_by_id(pid).await {
                curr_parent = parent.parent_id;
            } else {
                break;
            }
        }
        Ok(depth)
    }

    /// Root ancestor comment in conversation thread
    async fn root_comment(&self, ctx: &Context<'_>) -> Result<Option<CommentGql>> {
        let Some(mut pid) = self.0.parent_id else {
            return Ok(None);
        };
        let db = ctx.data::<Database>()?;
        let mut root = None;
        let mut depth = 0;
        while depth < 20 {
            if let Some(parent) = db.get_comment_by_id(pid).await {
                if let Some(next_pid) = parent.parent_id {
                    pid = next_pid;
                    depth += 1;
                } else {
                    root = Some(parent);
                    break;
                }
            } else {
                break;
            }
        }
        Ok(root.map(CommentGql))
    }

    async fn replies(&self, ctx: &Context<'_>) -> Result<Vec<CommentGql>> {
        let db = ctx.data::<Database>()?;
        let replies = db.get_replies_for_comment(self.0.id).await;
        Ok(replies.into_iter().map(CommentGql).collect())
    }

    /// Cursor-paginated threaded replies under this comment
    async fn replies_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<CommentConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (replies, has_next_page) = db
            .get_replies_cursor(self.0.id, page_size, cursor_pair)
            .await;
        let start_cursor = replies.first().map(|c| encode_cursor(c.created_at, c.id));
        let end_cursor = replies.last().map(|c| encode_cursor(c.created_at, c.id));

        let edges = replies
            .into_iter()
            .map(|c| {
                let cursor = encode_cursor(c.created_at, c.id);
                CommentEdgeGql {
                    cursor,
                    node: CommentGql(c),
                }
            })
            .collect();

        Ok(CommentConnectionGql {
            edges,
            page_info: PageInfoGql {
                has_next_page,
                has_previous_page: after.is_some(),
                start_cursor,
                end_cursor,
            },
            total_count: None,
        })
    }

    async fn replies_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentRepliesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_replies_count(self.0.id).await)
        }
    }

    /// Comment Likes Count (batch loaded via DataLoader)
    async fn likes_count(&self, ctx: &Context<'_>) -> Result<usize> {
        if let Some(loader) = ctx.data_opt::<DataLoader<CommentLikesCountLoader>>() {
            let count = loader.load_one(self.0.id).await?.unwrap_or(0);
            Ok(count)
        } else {
            let db = ctx.data::<Database>()?;
            Ok(db.get_comment_likes_count(self.0.id).await)
        }
    }

    /// Whether current viewer liked this comment
    async fn is_liked_by_me(&self, ctx: &Context<'_>) -> Result<bool> {
        if let Some(auth_user) = ctx.data_opt::<AuthUser>() {
            let db = ctx.data::<Database>()?;
            Ok(db.is_comment_liked_by(self.0.id, auth_user.user_id).await)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone)]
pub struct CommentEdgeGql {
    pub cursor: String,
    pub node: CommentGql,
}

#[Object]
impl CommentEdgeGql {
    async fn cursor(&self) -> &str {
        &self.cursor
    }

    async fn node(&self) -> &CommentGql {
        &self.node
    }
}

#[derive(Clone)]
pub struct CommentConnectionGql {
    pub edges: Vec<CommentEdgeGql>,
    pub page_info: PageInfoGql,
    pub total_count: Option<usize>,
}

#[Object]
impl CommentConnectionGql {
    async fn edges(&self) -> &[CommentEdgeGql] {
        &self.edges
    }

    async fn page_info(&self) -> &PageInfoGql {
        &self.page_info
    }

    async fn total_count(&self) -> Option<usize> {
        self.total_count
    }
}
