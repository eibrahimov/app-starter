-- Second example resource: posts with a draft -> published -> archived
-- lifecycle. Demonstrates status columns, partial timestamps, and a
-- composite index for filtered, ordered queries.
CREATE TABLE posts (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    -- 'draft' | 'published' | 'archived'
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL,
    published_at TEXT
);

CREATE INDEX idx_posts_status_created_at ON posts (status, created_at);
