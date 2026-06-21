-- Enforce the post lifecycle vocabulary at the database boundary.
--
-- src/posts.rs decodes `status` straight into the `PostStatus` enum on every
-- read (list, get, stats), so a stored value outside the closed set fails loud
-- (a decode error -> 500) instead of being silently dropped. This migration
-- closes the matching gap on the write side: SQLite itself rejects an
-- out-of-vocabulary status, so no path -- API, seed, or out-of-band SQL -- can
-- persist a status the enum cannot represent.
--
-- SQLite cannot ALTER an existing column to add a CHECK, so this is the standard
-- table rebuild: create the constrained table, copy every row, drop the old one,
-- rename, and recreate the index. The column definitions are otherwise identical
-- to 20260613000001_create_posts.sql. NOT NULL still rejects NULL (a CHECK
-- passes on NULL), so the two together pin status to exactly the three values.
--
-- The INSERT ... SELECT enforces the new CHECK, so applying this against a
-- database that already holds an out-of-vocabulary row fails the migration --
-- the intended fail-loud behavior. Every write in this template binds its status
-- through PostStatus (create/publish/archive), so no such row exists here. A fork
-- that widened the vocabulary in app code before pulling this migration must
-- either extend the CHECK set below or normalize offending rows first (e.g. UPDATE
-- posts SET status = 'draft' WHERE status NOT IN ('draft','published','archived')).
CREATE TABLE posts_new (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published', 'archived')),
    created_at TEXT NOT NULL,
    published_at TEXT
);

INSERT INTO posts_new (id, title, body, status, created_at, published_at)
SELECT id, title, body, status, created_at, published_at FROM posts;

DROP TABLE posts;

ALTER TABLE posts_new RENAME TO posts;

CREATE INDEX idx_posts_status_created_at ON posts (status, created_at);
