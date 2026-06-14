-- Single-row application settings. The row id is always 'app'. Seeded here so
-- the app has a base currency on first run; the user can change it in Settings.
CREATE TABLE settings (
    id TEXT PRIMARY KEY,
    base_currency TEXT NOT NULL DEFAULT 'USD',
    updated_at TEXT NOT NULL
);

INSERT INTO settings (id, base_currency, updated_at)
VALUES ('app', 'USD', '2026-06-14T00:00:00Z');
