-- Advanced Direct Messaging: Group chats, message edits, conversations
CREATE TABLE IF NOT EXISTS conversations
(
    id         UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    is_group   BOOLEAN     NOT NULL DEFAULT FALSE,
    title      VARCHAR(100),
    created_by UUID        REFERENCES users (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS conversation_participants
(
    conversation_id UUID        NOT NULL REFERENCES conversations (id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_read_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_conv_participants_user ON conversation_participants (user_id);

ALTER TABLE direct_messages
    ADD COLUMN IF NOT EXISTS conversation_id UUID REFERENCES conversations (id) ON DELETE CASCADE;
ALTER TABLE direct_messages
    ADD COLUMN IF NOT EXISTS is_edited BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX IF NOT EXISTS idx_direct_messages_conversation ON direct_messages (conversation_id, created_at ASC);
