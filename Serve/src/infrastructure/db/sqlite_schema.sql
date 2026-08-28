-- SQLite Database Schema for Serve

CREATE TABLE IF NOT EXISTS users
(
    id               TEXT PRIMARY KEY,
    username         TEXT UNIQUE NOT NULL,
    email            TEXT UNIQUE NOT NULL,
    password_hash    TEXT        NOT NULL,
    display_name     TEXT        NOT NULL,
    bio              TEXT,
    avatar_url       TEXT,
    header_image_url TEXT,
    location         TEXT,
    website          TEXT,
    is_private       BOOLEAN     NOT NULL DEFAULT 0,
    is_2fa_enabled   BOOLEAN     NOT NULL DEFAULT 0,
    totp_secret      TEXT,
    created_at       TEXT        NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS posts
(
    id                TEXT PRIMARY KEY,
    author_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content           TEXT    NOT NULL,
    quote_post_id     TEXT    REFERENCES posts (id) ON DELETE SET NULL,
    pinned_comment_id TEXT,
    audience          TEXT    NOT NULL DEFAULT 'PUBLIC',
    views_count       INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS post_media
(
    id         TEXT PRIMARY KEY,
    post_id    TEXT    NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    media_url  TEXT    NOT NULL,
    media_type TEXT    NOT NULL DEFAULT 'IMAGE',
    alt_text   TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    width      INTEGER,
    height     INTEGER,
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS comments
(
    id         TEXT PRIMARY KEY,
    post_id    TEXT    NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    author_id  TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    parent_id  TEXT REFERENCES comments (id) ON DELETE CASCADE,
    content    TEXT    NOT NULL,
    is_edited  BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS follows
(
    follower_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    followee_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (follower_id, followee_id)
);

CREATE TABLE IF NOT EXISTS likes
(
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, post_id)
);

CREATE TABLE IF NOT EXISTS reposts
(
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, post_id)
);

CREATE TABLE IF NOT EXISTS bookmarks
(
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, post_id)
);

CREATE TABLE IF NOT EXISTS comment_likes
(
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    comment_id TEXT NOT NULL REFERENCES comments (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, comment_id)
);

CREATE TABLE IF NOT EXISTS stories
(
    id         TEXT PRIMARY KEY,
    author_id  TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    media_url  TEXT NOT NULL,
    caption    TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS story_views
(
    story_id  TEXT NOT NULL REFERENCES stories (id) ON DELETE CASCADE,
    viewer_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    viewed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (story_id, viewer_id)
);

CREATE TABLE IF NOT EXISTS conversations
(
    id         TEXT PRIMARY KEY,
    is_group   BOOLEAN NOT NULL DEFAULT 0,
    title      TEXT,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS conversation_participants
(
    conversation_id TEXT NOT NULL REFERENCES conversations (id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    joined_at       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_read_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (conversation_id, user_id)
);

CREATE TABLE IF NOT EXISTS direct_messages
(
    id              TEXT PRIMARY KEY,
    sender_id       TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    recipient_id    TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    conversation_id TEXT REFERENCES conversations (id) ON DELETE CASCADE,
    content         TEXT    NOT NULL,
    is_read         BOOLEAN NOT NULL DEFAULT 0,
    is_edited       BOOLEAN NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS notifications
(
    id                TEXT PRIMARY KEY,
    recipient_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    actor_id          TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    notification_type TEXT    NOT NULL,
    entity_id         TEXT,
    is_read           BOOLEAN NOT NULL DEFAULT 0,
    created_at        TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS blocks
(
    blocker_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    blocked_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (blocker_id, blocked_id)
);

CREATE TABLE IF NOT EXISTS mutes
(
    muter_id   TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    muted_id   TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (muter_id, muted_id)
);

CREATE TABLE IF NOT EXISTS follow_requests
(
    id           TEXT PRIMARY KEY,
    requester_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    status       TEXT NOT NULL DEFAULT 'PENDING',
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (requester_id, target_id)
);

CREATE TABLE IF NOT EXISTS hashtags
(
    id          TEXT PRIMARY KEY,
    name        TEXT UNIQUE NOT NULL,
    posts_count INTEGER     NOT NULL DEFAULT 0,
    created_at  TEXT        NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS post_hashtags
(
    post_id    TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    hashtag_id TEXT NOT NULL REFERENCES hashtags (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (post_id, hashtag_id)
);

CREATE TABLE IF NOT EXISTS user_mentions
(
    post_id    TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (post_id, user_id)
);

CREATE TABLE IF NOT EXISTS polls
(
    id         TEXT PRIMARY KEY,
    post_id    TEXT NOT NULL UNIQUE REFERENCES posts (id) ON DELETE CASCADE,
    question   TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS poll_options
(
    id          TEXT PRIMARY KEY,
    poll_id     TEXT    NOT NULL REFERENCES polls (id) ON DELETE CASCADE,
    option_text TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS poll_votes
(
    poll_id    TEXT NOT NULL REFERENCES polls (id) ON DELETE CASCADE,
    option_id  TEXT NOT NULL REFERENCES poll_options (id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (poll_id, user_id)
);

CREATE TABLE IF NOT EXISTS user_lists
(
    id          TEXT PRIMARY KEY,
    owner_id    TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    description TEXT,
    is_private  BOOLEAN NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_list_members
(
    list_id    TEXT NOT NULL REFERENCES user_lists (id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (list_id, user_id)
);

CREATE TABLE IF NOT EXISTS bookmark_collections
(
    id          TEXT PRIMARY KEY,
    user_id     TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    description TEXT,
    is_private  BOOLEAN NOT NULL DEFAULT 1,
    created_at  TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS collection_bookmarks
(
    collection_id TEXT NOT NULL REFERENCES bookmark_collections (id) ON DELETE CASCADE,
    post_id       TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_id, post_id)
);

CREATE TABLE IF NOT EXISTS reports
(
    id          TEXT PRIMARY KEY,
    reporter_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_type TEXT NOT NULL,
    target_id   TEXT NOT NULL,
    reason      TEXT NOT NULL,
    details     TEXT,
    status      TEXT NOT NULL DEFAULT 'PENDING',
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS post_views
(
    post_id   TEXT NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    viewer_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    viewed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (post_id, viewer_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_posts_author_id ON posts (author_id);
CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments (post_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_id ON comments (author_id);
CREATE INDEX IF NOT EXISTS idx_follows_follower ON follows (follower_id);
CREATE INDEX IF NOT EXISTS idx_follows_followee ON follows (followee_id);
CREATE INDEX IF NOT EXISTS idx_likes_post ON likes (post_id);
CREATE INDEX IF NOT EXISTS idx_likes_user ON likes (user_id);
CREATE INDEX IF NOT EXISTS idx_stories_author ON stories (author_id);
CREATE INDEX IF NOT EXISTS idx_dm_recipient ON direct_messages (recipient_id);
CREATE INDEX IF NOT EXISTS idx_dm_sender ON direct_messages (sender_id);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications (recipient_id);
