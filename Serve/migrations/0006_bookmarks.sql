-- Bookmarks / Saved posts table
CREATE TABLE IF NOT EXISTS bookmarks
(
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, post_id)
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_user_created ON bookmarks (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_post_user ON bookmarks (post_id, user_id);
