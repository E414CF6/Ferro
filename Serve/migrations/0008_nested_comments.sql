-- Nested threaded comments support
ALTER TABLE comments
    ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES comments (id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON comments (parent_id);
CREATE INDEX IF NOT EXISTS idx_comments_post_parent ON comments (post_id, parent_id, created_at ASC);
