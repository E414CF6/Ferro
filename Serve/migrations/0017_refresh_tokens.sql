-- Migration 0017: Refresh Tokens and Session Management for Dual Token (Access + Refresh) RTR
CREATE TABLE IF NOT EXISTS refresh_tokens
(
    id
    UUID
    PRIMARY
    KEY,
    user_id
    UUID
    NOT
    NULL
    REFERENCES
    users
(
    id
) ON DELETE CASCADE,
    token_hash VARCHAR
(
    64
) NOT NULL UNIQUE,
    jti VARCHAR
(
    64
) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens (token_hash);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_jti ON refresh_tokens (jti);
