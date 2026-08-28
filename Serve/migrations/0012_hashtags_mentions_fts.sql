-- Hashtags, User Mentions & Full-Text Search indexing
CREATE TABLE IF NOT EXISTS hashtags
(
    id          UUID PRIMARY KEY             DEFAULT gen_random_uuid(),
    name        VARCHAR(100) UNIQUE NOT NULL,
    posts_count BIGINT              NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hashtags_name ON hashtags (name);
CREATE INDEX IF NOT EXISTS idx_hashtags_posts_count ON hashtags (posts_count DESC);

CREATE TABLE IF NOT EXISTS post_hashtags
(
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    hashtag_id UUID        NOT NULL REFERENCES hashtags (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, hashtag_id)
);

CREATE INDEX IF NOT EXISTS idx_post_hashtags_hashtag ON post_hashtags (hashtag_id, created_at DESC);

CREATE TABLE IF NOT EXISTS user_mentions
(
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_user_mentions_user ON user_mentions (user_id);
