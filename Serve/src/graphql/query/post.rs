use crate::application::AppServices;
use crate::application::helpers::resolve_user_id;
use crate::graphql::types::{
    HashtagTrendGql, PageInfoGql, PostAnalyticsGql, PostConnectionGql, PostEdgeGql, PostGql,
    decode_cursor, encode_cursor,
};
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct PostQuery;

#[Object]
impl PostQuery {
    /// Get single post by ID
    async fn post(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PostGql>> {
        let services = ctx.data::<AppServices>()?;
        let pid = Uuid::parse_str(&id)?;
        let post = services
            .post
            .get_post_by_id(pid)
            .await
            .map_err(|e| e.extend())?;
        Ok(post.map(PostGql))
    }

    /// Get post analytics and engagement metrics (strictly restricted to author)
    async fn post_analytics(&self, ctx: &Context<'_>, post_id: ID) -> Result<PostAnalyticsGql> {
        let auth_uid = crate::application::helpers::require_auth(ctx)?;
        let services = ctx.data::<AppServices>()?;
        let pid = Uuid::parse_str(&post_id)?;
        let an = services
            .post
            .get_post_analytics(pid, auth_uid)
            .await
            .map_err(|e| e.extend())?;
        Ok(PostAnalyticsGql(an))
    }

    /// List all posts with offset pagination
    async fn posts(
        &self,
        ctx: &Context<'_>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<PostGql>> {
        let services = ctx.data::<AppServices>()?;
        let posts = services
            .post
            .get_posts(limit, offset)
            .await
            .map_err(|e| e.extend())?;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// List all posts with cursor-based pagination
    async fn posts_connection(
        &self,
        ctx: &Context<'_>,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = services
            .post
            .get_posts_cursor(page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
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
        let services = ctx.data::<AppServices>()?;
        let posts = services
            .post
            .get_feed(uid, limit, offset)
            .await
            .map_err(|e| e.extend())?;
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
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = services
            .post
            .get_feed_cursor(uid, page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
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
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = services
            .bookmark
            .get_saved_posts_cursor(uid, page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
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

    /// Get posts saved / bookmarked by current user (flat list)
    async fn saved_posts(&self, ctx: &Context<'_>, user_id: Option<ID>) -> Result<Vec<PostGql>> {
        let uid = resolve_user_id(ctx, user_id)?;
        let services = ctx.data::<AppServices>()?;
        let (posts, _) = services
            .bookmark
            .get_saved_posts_cursor(uid, 50, None)
            .await
            .map_err(|e| e.extend())?;
        Ok(posts.into_iter().map(PostGql).collect())
    }

    /// Search posts containing full-text keyword with cursor pagination
    async fn search_posts_connection(
        &self,
        ctx: &Context<'_>,
        query: String,
        first: Option<usize>,
        after: Option<String>,
    ) -> Result<PostConnectionGql> {
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = services
            .discovery
            .search_posts_cursor(&query, page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
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
        let services = ctx.data::<AppServices>()?;
        let page_size = first.unwrap_or(20).clamp(1, 100);
        let cursor_pair = if let Some(ref a) = after {
            decode_cursor(a)
        } else {
            None
        };

        let (posts, has_next_page) = services
            .post
            .get_posts_by_hashtag_cursor(&hashtag, page_size, cursor_pair)
            .await
            .map_err(|e| e.extend())?;
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
        let services = ctx.data::<AppServices>()?;
        let list = services
            .discovery
            .get_trending_hashtags(limit)
            .await
            .map_err(|e| e.extend())?;
        Ok(list
            .into_iter()
            .map(|(t, c)| HashtagTrendGql { tag: t, count: c })
            .collect())
    }
}
