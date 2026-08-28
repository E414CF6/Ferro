-- Migration 0014: Advanced Social Platform Features
-- 1. Post Media Attachments
CREATE TABLE IF NOT EXISTS post_media
(
    id         UUID PRIMARY KEY,
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    media_url  TEXT        NOT NULL,
    media_type VARCHAR(20) NOT NULL DEFAULT 'IMAGE', -- IMAGE, VIDEO, GIF
    alt_text   TEXT,
    sort_order INT         NOT NULL DEFAULT 0,
    width      INT,
    height     INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_post_media_post ON post_media (post_id, sort_order ASC);

-- 2. Interactive Polls & Voting
CREATE TABLE IF NOT EXISTS polls
(
    id         UUID PRIMARY KEY,
    post_id    UUID        NOT NULL UNIQUE REFERENCES posts (id) ON DELETE CASCADE,
    question   TEXT        NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_polls_post ON polls (post_id);

CREATE TABLE IF NOT EXISTS poll_options
(
    id          UUID PRIMARY KEY,
    poll_id     UUID NOT NULL REFERENCES polls (id) ON DELETE CASCADE,
    option_text TEXT NOT NULL,
    sort_order  INT  NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_poll_options_poll ON poll_options (poll_id, sort_order ASC);

CREATE TABLE IF NOT EXISTS poll_votes
(
    poll_id    UUID        NOT NULL REFERENCES polls (id) ON DELETE CASCADE,
    option_id  UUID        NOT NULL REFERENCES poll_options (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (poll_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_poll_votes_option ON poll_votes (option_id);

-- 3. Custom User Lists & Post Audience
CREATE TABLE IF NOT EXISTS user_lists
(
    id          UUID PRIMARY KEY,
    owner_id    UUID         NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    is_private  BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_user_lists_owner ON user_lists (owner_id);

CREATE TABLE IF NOT EXISTS user_list_members
(
    list_id    UUID        NOT NULL REFERENCES user_lists (id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (list_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_user_list_members_user ON user_list_members (user_id);

ALTER TABLE posts
    ADD COLUMN IF NOT EXISTS audience VARCHAR(30) NOT NULL DEFAULT 'PUBLIC';

-- 4. Bookmark Collections
CREATE TABLE IF NOT EXISTS bookmark_collections
(
    id          UUID PRIMARY KEY,
    user_id     UUID         NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    is_private  BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_bookmark_collections_user ON bookmark_collections (user_id);

CREATE TABLE IF NOT EXISTS collection_bookmarks
(
    collection_id UUID        NOT NULL REFERENCES bookmark_collections (id) ON DELETE CASCADE,
    post_id       UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (collection_id, post_id)
);
CREATE INDEX IF NOT EXISTS idx_collection_bookmarks_post ON collection_bookmarks (post_id);

-- 5. Content Reporting & Moderation
CREATE TABLE IF NOT EXISTS reports
(
    id          UUID PRIMARY KEY,
    reporter_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_type VARCHAR(20) NOT NULL,                   -- POST, COMMENT, USER
    target_id   UUID        NOT NULL,
    reason      VARCHAR(30) NOT NULL,                   -- SPAM, HARASSMENT, HATE_SPEECH, INAPPROPRIATE, COPYRIGHT, OTHER
    details     TEXT,
    status      VARCHAR(20) NOT NULL DEFAULT 'PENDING', -- PENDING, RESOLVED, DISMISSED
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_reports_target ON reports (target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_reports_status ON reports (status);

-- 6. Post Views & Analytics
ALTER TABLE posts
    ADD COLUMN IF NOT EXISTS views_count BIGINT NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS post_views
(
    post_id   UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    viewer_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, viewer_id)
);
CREATE INDEX IF NOT EXISTS idx_post_views_post ON post_views (post_id);

-- 7. 2FA (Two-Factor Authentication)
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS totp_secret VARCHAR(64);
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS is_2fa_enabled BOOLEAN NOT NULL DEFAULT FALSE;
