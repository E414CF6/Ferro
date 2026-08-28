CREATE TABLE IF NOT EXISTS stories
(
    id         UUID PRIMARY KEY,
    author_id  UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    media_url  TEXT        NOT NULL,
    caption    TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

CREATE TABLE IF NOT EXISTS story_views
(
    story_id  UUID        NOT NULL REFERENCES stories (id) ON DELETE CASCADE,
    viewer_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (story_id, viewer_id)
);
