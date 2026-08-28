-- Notifications table for real-time interaction alerts
CREATE TABLE IF NOT EXISTS notifications
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    recipient_id      UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    actor_id          UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL,
    entity_id         UUID,
    is_read           BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_recipient_created ON notifications (recipient_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient_is_read ON notifications (recipient_id, is_read);
