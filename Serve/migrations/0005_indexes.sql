-- Secondary performance indexes for queries & foreign key joins
CREATE INDEX IF NOT EXISTS idx_posts_author_created ON posts (author_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comments_post_created ON comments (post_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_comments_author_created ON comments (author_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_likes_post_user ON likes (post_id, user_id);
CREATE INDEX IF NOT EXISTS idx_likes_user_created ON likes (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_follows_followee ON follows (followee_id, follower_id);
CREATE INDEX IF NOT EXISTS idx_follows_follower ON follows (follower_id, followee_id);
CREATE INDEX IF NOT EXISTS idx_stories_active ON stories (expires_at DESC, author_id);
CREATE INDEX IF NOT EXISTS idx_stories_author ON stories (author_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_story_views_story ON story_views (story_id, viewer_id);
CREATE INDEX IF NOT EXISTS idx_story_views_viewer ON story_views (viewer_id, viewed_at DESC);
