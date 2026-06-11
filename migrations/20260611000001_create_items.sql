-- Example resource. Replace with your real schema; keep the file naming
-- convention (<timestamp>_<description>.sql) so sqlx orders migrations.
CREATE TABLE items (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_items_created_at ON items (created_at);
