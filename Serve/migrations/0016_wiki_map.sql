-- 0016_wiki_map.sql
-- Migration for wMap (Wiki Map) integration

CREATE TABLE IF NOT EXISTS wiki_articles
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users (id) ON DELETE SET NULL,
    title       VARCHAR(255)     NOT NULL,
    slug        VARCHAR(255)     NOT NULL UNIQUE,
    summary     TEXT,
    content     TEXT             NOT NULL,
    latitude    DOUBLE PRECISION NOT NULL,
    longitude   DOUBLE PRECISION NOT NULL,
    zoom        DOUBLE PRECISION NOT NULL DEFAULT 14.0,
    category    VARCHAR(100)     NOT NULL DEFAULT '장소',
    tags        TEXT             NOT NULL DEFAULT '[]',
    geojson     TEXT,
    author      VARCHAR(100)     NOT NULL DEFAULT '익명',
    views       BIGINT           NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ      NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ      NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS wiki_revisions
(
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id   UUID             NOT NULL REFERENCES wiki_articles (id) ON DELETE CASCADE,
    user_id      UUID REFERENCES users (id) ON DELETE SET NULL,
    title        VARCHAR(255)     NOT NULL,
    content      TEXT             NOT NULL,
    latitude     DOUBLE PRECISION NOT NULL,
    longitude    DOUBLE PRECISION NOT NULL,
    edit_summary VARCHAR(255),
    author       VARCHAR(100)     NOT NULL DEFAULT '익명',
    created_at   TIMESTAMPTZ      NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS wiki_view_logs
(
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID        NOT NULL REFERENCES wiki_articles (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance and geolocation querying
CREATE INDEX IF NOT EXISTS idx_wiki_articles_lat_lng ON wiki_articles (latitude, longitude);
CREATE INDEX IF NOT EXISTS idx_wiki_articles_category ON wiki_articles (category);
CREATE INDEX IF NOT EXISTS idx_wiki_articles_slug ON wiki_articles (slug);
CREATE INDEX IF NOT EXISTS idx_wiki_articles_views ON wiki_articles (views DESC);
CREATE INDEX IF NOT EXISTS idx_wiki_articles_updated_at ON wiki_articles (updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_wiki_articles_user_id ON wiki_articles (user_id);

CREATE INDEX IF NOT EXISTS idx_wiki_revisions_article_id ON wiki_revisions (article_id);
CREATE INDEX IF NOT EXISTS idx_wiki_revisions_created_at ON wiki_revisions (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_wiki_view_logs_article_created ON wiki_view_logs (article_id, created_at);
CREATE INDEX IF NOT EXISTS idx_wiki_view_logs_created_at ON wiki_view_logs (created_at);
