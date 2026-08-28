-- Reposts and Quote Posts support
CREATE TABLE IF NOT EXISTS reposts
(
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, post_id)
);

CREATE INDEX IF NOT EXISTS idx_reposts_user ON reposts (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_reposts_post ON reposts (post_id);

ALTER TABLE posts
    ADD COLUMN IF NOT EXISTS quote_post_id UUID REFERENCES posts (id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_posts_quote_post_id ON posts (quote_post_id);
