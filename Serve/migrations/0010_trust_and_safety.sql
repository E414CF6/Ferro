-- Trust & Safety: Blocks, Mutes, Private Accounts & Follow Requests
CREATE TABLE IF NOT EXISTS blocks
(
    blocker_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    blocked_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (blocker_id, blocked_id)
);

CREATE INDEX IF NOT EXISTS idx_blocks_blocker ON blocks (blocker_id);
CREATE INDEX IF NOT EXISTS idx_blocks_blocked ON blocks (blocked_id);

CREATE TABLE IF NOT EXISTS mutes
(
    muter_id   UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    muted_id   UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (muter_id, muted_id)
);

CREATE INDEX IF NOT EXISTS idx_mutes_muter ON mutes (muter_id);

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS is_private BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS follow_requests
(
    id           UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    requester_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    status       VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (requester_id, target_id)
);

CREATE INDEX IF NOT EXISTS idx_follow_requests_target ON follow_requests (target_id, status);
