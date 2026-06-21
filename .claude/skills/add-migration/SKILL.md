---
name: add-migration
description: >-
  Add a sqlx migration that evolves an EXISTING table in this app-starter template
  (add a column, index, constraint, or backfill) without breaking sqlx's append-only
  checksum rule. Use when asked to change or alter a table, add a column or index, write
  a migration, or evolve the schema for a resource that already exists. For a brand-new
  resource wired through to the UI, use the add-resource skill instead. Runs a read-only
  check that catches edited or renamed committed migrations -- the #1 migration footgun.
---

# Add a migration

Operational procedure for evolving existing schema. Canonical detail lives in:

- [`migrations/README.md`](../../../migrations/README.md) (the append-only rule + conventions)
- [`AGENTS.md`](../../../AGENTS.md) -> "Adding a resource end to end" (full field conventions)

> For a whole new resource (table + API + UI), use **add-resource**. This skill is for
> changing a table that already exists.

## The one rule that matters

**Migrations are append-only. NEVER edit or rename a committed migration.** sqlx stores a
checksum of every applied file; changing one makes the app refuse to start against any
database that already ran it. There are no down-migrations — to fix a bad migration, add a
new corrective forward migration.

## Procedure

1. **Create a new file** `migrations/<YYYYMMDDHHMMSS>_<description>.sql` whose timestamp
   sorts after every existing migration. Match the naming of the files already there.
2. **Write the forward change** only (e.g. `ALTER TABLE ... ADD COLUMN ...`,
   `CREATE INDEX ...`). Follow the conventions: TEXT uuid keys, INTEGER booleans (0/1),
   TEXT RFC-3339 timestamps, an index on any column you sort or filter by. Give new columns
   a default or backfill them so existing rows stay valid. SQLite cannot `ALTER` a column
   to add a `CHECK` or change its type — use the table-rebuild pattern (create `*_new`,
   `INSERT ... SELECT`, `DROP`, `RENAME`, recreate indexes); see
   `migrations/20260621000001_add_posts_status_check.sql` for a worked example.
3. **Reflect it in code** if the shape changed: update the row struct in `src/<resource>.rs`
   and the `SELECT` columns; if the API response changed, update `#[utoipa::path]`/schemas,
   run `just typegen`, and commit `interface/src/api/schema.d.ts` (additive within `/api/v1`).
4. **Check** (read-only) that you did not touch a committed migration and the new file sorts
   last:
   ```sh
   .claude/skills/add-migration/scripts/check-migrations.sh
   ```
5. **Validate:** `just test` (migrations run against in-memory SQLite), then `just verify`.

## Stop and get human approval before (per AGENTS.md)

Anything touching migration history, data-destructive changes, or editing/renaming a
committed migration is approval-gated. Removing or repurposing an `/api/v1` field is a
breaking change — open `/api/v2` instead.
