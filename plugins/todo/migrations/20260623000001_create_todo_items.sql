-- Plugin-owned migration. Runs into the `_sqlx_migrations_todo` tracking table
-- (the host sets that name), so this plugin owns an independent version keyspace.
-- Table is prefixed with the plugin name (`todo_*`) per docs/plugin-framework.md §5.
CREATE TABLE todo_items (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_todo_items_created_at ON todo_items (created_at);
