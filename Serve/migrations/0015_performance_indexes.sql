-- High-performance composite indexes for keyset pagination, data loader batching, and high-concurrency feeds

CREATE INDEX IF NOT EXISTS idx_posts_author_created ON posts (author_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_posts_created_at_id ON posts (created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_comments_post_created ON comments (post_id, created_at ASC, id ASC);
CREATE INDEX IF NOT EXISTS idx_comments_parent_created ON comments (parent_id, created_at ASC, id ASC);
CREATE INDEX IF NOT EXISTS idx_post_media_post_sort ON post_media (post_id, sort_order ASC);
CREATE INDEX IF NOT EXISTS idx_poll_options_poll_sort ON poll_options (poll_id, sort_order ASC);
CREATE INDEX IF NOT EXISTS idx_poll_votes_poll_user ON poll_votes (poll_id, user_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_option ON poll_votes (option_id);
CREATE INDEX IF NOT EXISTS idx_user_lists_owner_created ON user_lists (owner_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_list_members_list_created ON user_list_members (list_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_user_list_members_user ON user_list_members (user_id);
CREATE INDEX IF NOT EXISTS idx_bookmark_collections_user_created ON bookmark_collections (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_collection_bookmarks_coll_created ON collection_bookmarks (collection_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_reports_status_created ON reports (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_reports_target ON reports (target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_post_views_post ON post_views (post_id);
CREATE INDEX IF NOT EXISTS idx_comment_likes_comment ON comment_likes (comment_id);
