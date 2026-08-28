CREATE TABLE IF NOT EXISTS direct_messages
(
    id           UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    sender_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    recipient_id UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content      TEXT        NOT NULL,
    is_read      BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_direct_messages_sender_recipient ON direct_messages (sender_id, recipient_id, created_at);
CREATE INDEX IF NOT EXISTS idx_direct_messages_recipient_is_read ON direct_messages (recipient_id, is_read);
CREATE INDEX IF NOT EXISTS idx_direct_messages_created_at ON direct_messages (created_at DESC);
