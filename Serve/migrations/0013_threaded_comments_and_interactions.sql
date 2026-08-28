-- Threaded comments interactions, pinned comments, comment likes, and edits
CREATE TABLE IF NOT EXISTS comment_likes
(
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    comment_id UUID        NOT NULL REFERENCES comments (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, comment_id)
);

CREATE INDEX IF NOT EXISTS idx_comment_likes_comment ON comment_likes (comment_id);
CREATE INDEX IF NOT EXISTS idx_comment_likes_user ON comment_likes (user_id);

ALTER TABLE posts
    ADD COLUMN IF NOT EXISTS pinned_comment_id UUID REFERENCES comments (id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_posts_pinned_comment ON posts (pinned_comment_id);

ALTER TABLE comments
    ADD COLUMN IF NOT EXISTS is_edited BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE comments
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
