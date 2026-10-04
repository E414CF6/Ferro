use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::BookmarkCollection as BookmarkCollectionModel;
use crate::domain::repositories::*;
use crate::infrastructure::db::loaders::UserLoader;
use crate::infrastructure::db::postgres::Database;

use super::{
    decode_cursor, encode_cursor, PageInfoGql, PostConnectionGql, PostEdgeGql, PostGql, UserGql,
};

#[derive(Clone)]
pub struct BookmarkCollectionGql(pub BookmarkCollectionModel);

#[Object]
impl BookmarkCollectionGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.0.name
    }

    async fn description(&self) -> Option<&str> {
        self.0.description.as_deref()
    }

    async fn is_private(&self) -> bool {
        self.0.is_private
    }

    async fn user(&self, ctx: &Context<'_>) -> Result<UserGql> {
        if let Some(loader) = ctx.data_opt::<DataLoader<UserLoader>>() {
            let user = loader.load_one(self.0.user_id).await?.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;
            Ok(UserGql(user))
        } else {
            let db = ctx.data::<Database>()?;
            let user = db.get_user_by_id(self.0.user_id).await.ok_or_else(|| {
                DomainError::new(ErrorCode::UserNotFound, "User not found").extend()
            })?;
            Ok(UserGql(user))
        }
    }

    async fn posts_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db
            .get_collection_posts_cursor(self.0.id, page_size, cursor_pair)
            .await;
        let start_cursor = posts.first().map(|p| encode_cursor(p.created_at, p.id));
        let end_cursor = posts.last().map(|p| encode_cursor(p.created_at, p.id));

        let edges = posts
            .into_iter()
            .map(|p| {
                let cursor = encode_cursor(p.created_at, p.id);
                PostEdgeGql {
                    cursor,
                    node: PostGql(p),
                }
            })
            .collect();

        Ok(PostConnectionGql {
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
}
