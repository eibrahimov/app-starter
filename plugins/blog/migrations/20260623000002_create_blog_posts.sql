-- Plugin-owned migration (its own `_sqlx_migrations_blog` keyspace; the host sets
-- that name). Table is prefixed with the plugin name (`blog_*`) per §5.
--
-- This collapses the two historical core posts migrations (create + the CHECK
-- rebuild) into one: a fresh plugin migrator has no existing rows to preserve, so
-- the table is created with the lifecycle CHECK directly. The CHECK + NOT NULL
-- pin `status` to exactly the three values, matching the `PostStatus` enum the
-- plugin decodes on every read.
CREATE TABLE blog_posts (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published', 'archived')),
    created_at TEXT NOT NULL,
    published_at TEXT
);

CREATE INDEX idx_blog_posts_status_created_at ON blog_posts (status, created_at);
