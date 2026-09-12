use crate::application::helpers::resolve_user_id;
use crate::domain::repositories::{BookmarkRepository, PostRepository};
use crate::graphql::types::{
    HashtagTrendGql, PageInfoGql, PostAnalyticsGql, PostConnectionGql, PostEdgeGql, PostGql,
    decode_cursor, encode_cursor,
};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    /// Get single post by ID
    async fn post(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PostGql>> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&id)?;
        Ok(db.get_post_by_id(pid).await.map(PostGql))
    }

    /// Get post analytics and engagement metrics
    async fn post_analytics(&self, ctx: &Context<'_>, post_id: ID) -> Result<PostAnalyticsGql> {
        let db = ctx.data::<Database>()?;
        let pid = Uuid::parse_str(&post_id)?;
        let an = db.get_post_analytics(pid).await.map_err(|e| e.extend())?;
        Ok(PostAnalyticsGql(an))
    }

    /// List all posts with offset pagination
    async fn posts(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let db = ctx.data::<Database>()?;
        let posts = db.get_posts(limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// List all posts with cursor-based pagination
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

        let (posts, has_next_page) = db.get_posts_cursor(page_size, cursor_pair).await;
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

    /// Timeline feed with offset pagination
    async fn feed(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let posts = db.get_feed(uid, limit, offset).await;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// Timeline feed with cursor-based pagination
    async fn feed_connection(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_feed_cursor(uid, page_size, cursor_pair).await;
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

    /// Get posts saved / bookmarked by current user with cursor pagination
    async fn saved_posts_connection(
        &self,
        ctx: &Context<'_>,
        user_id: Option<ID>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let uid = resolve_user_id(ctx, user_id)?;
        let db = ctx.data::<Database>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = db.get_saved_posts_cursor(uid, page_size, cursor_pair).await;
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

    /// Search posts containing full-text keyword with cursor pagination
    async fn search_posts_connection(
        &self,
        ctx: &Context<'_>,
        query: String,
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

        let (posts, has_next_page) = db.search_posts_cursor(&query, page_size, cursor_pair).await;
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

    /// Get posts associated with a hashtag with cursor pagination
    async fn posts_by_hashtag(
        &self,
        ctx: &Context<'_>,
        hashtag: String,
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
            .get_posts_by_hashtag_cursor(&hashtag, page_size, cursor_pair)
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

    /// Get trending hashtags
    async fn trending_hashtags(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
    ) -> Result<Vec<HashtagTrendGql>> {
        let db = ctx.data::<Database>()?;
        let list = db.get_trending_hashtags(limit).await;
        Ok(list
            .into_iter()
            .map(|(t, c)| HashtagTrendGql { tag: t, count: c })
            .collect())
    }
}
