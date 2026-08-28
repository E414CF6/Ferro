CREATE TABLE IF NOT EXISTS users
(
    id               UUID PRIMARY KEY,
    username         VARCHAR(100) UNIQUE NOT NULL,
    email            VARCHAR(255) UNIQUE NOT NULL,
    password_hash    VARCHAR(255)        NOT NULL,
    display_name     VARCHAR(100)        NOT NULL,
    bio              TEXT,
    avatar_url       TEXT,
    header_image_url TEXT,
    location         VARCHAR(100),
    website          VARCHAR(255),
    created_at       TIMESTAMPTZ         NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS posts
(
    id         UUID PRIMARY KEY,
    author_id  UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content    TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS comments
(
    id         UUID PRIMARY KEY,
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    author_id  UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content    TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS follows
(
    follower_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    followee_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_id, followee_id)
);

CREATE TABLE IF NOT EXISTS likes
(
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    post_id    UUID        NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, post_id)
);
