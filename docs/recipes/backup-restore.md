# SQLite backup, restore, and integrity

App Starter stores data in a single SQLite file. This recipe gives you a safe,
schema-agnostic way to snapshot it, put a snapshot back, and verify a database is
healthy — covering `items`, `posts`, and any table you add. It is the concrete
answer to the production-readiness items "define backup and restore procedures"
and "test restore on a throwaway environment" (see
[production-readiness.md](../production-readiness.md)).

Three `just` recipes, backed by [`scripts/db.sh`](../../scripts/db.sh):

| Recipe | What it does |
| --- | --- |
| `just db-backup` | Online snapshot of the live database to a timestamped file in `backups/`. |
| `just db-restore <file>` | Restore from a snapshot. Guarded — confirms before overwriting and snapshots the current database first. |
| `just db-check` | `PRAGMA integrity_check` plus a list of applied migrations. Non-zero exit on failure, so it is usable in CI and scripts. |

Requires the `sqlite3` CLI on your PATH (macOS ships it; Debian/Ubuntu:
`apt-get install sqlite3`). No `just`? Run the underlying script directly:
`bash scripts/db.sh {backup|restore <file>|check}`.

Verify the helper itself with `just db-selftest` (or `bash scripts/db.test.sh`),
which runs the full backup -> restore -> check round-trip and the guard rails
against a throwaway database in a temp dir — it never touches your real data.

## Why not `cp`?

The server (via sqlx) opens SQLite in **WAL** mode, so a live database is spread
across three files: `app.db`, `app.db-wal`, and `app.db-shm`. A plain
`cp app.db backup.db` of the main file alone can capture a half-written page or
miss committed transactions still in the `-wal` tail — a subtly corrupt backup
that looks fine until you restore it.

`db-backup` uses SQLite's **online backup API** (`sqlite3 .backup`), which takes a
transaction-consistent snapshot of a live single-writer database into one
self-contained file, then verifies that snapshot with `integrity_check` before
reporting success. Never `cp` a database the server might be writing to.

## Where the database lives

The recipes derive the target file from `DATABASE_URL` exactly as the server does
(see [`src/db.rs`](../../src/db.rs)): strip the `sqlite://` scheme, drop the
`?query`, and reject `:memory:`. Resolution order:

1. the `DATABASE_URL` environment variable, if set;
2. the `DATABASE_URL` line in `.env` (the same file the server loads at startup);
3. the built-in default `sqlite://data/app.db?mode=rwc`.

A relative path resolves against the repository root — the server's working
directory in the standard `cargo run` and Docker setups:

| `DATABASE_URL` | Resolved file |
| --- | --- |
| `sqlite://data/app.db?mode=rwc` (default) | `<repo>/data/app.db` |
| `sqlite://app.db?mode=rwc` | `<repo>/app.db` |
| `sqlite:///data/app.db?mode=rwc` (Docker, absolute) | `/data/app.db` |
| `sqlite::memory:` | none — backup/restore/check refuse, since there is no file |

Run these recipes from the repository root, or pass an absolute `DATABASE_URL`, so
a relative path resolves where you expect. A desktop install's database is not
under the repo — it lives in the OS application-data directory (see
[Desktop](#desktop)), so point `DATABASE_URL` at that absolute path.

## Backup

```bash
just db-backup
# Backed up /repo/data/app.db -> /repo/backups/app-20260621T150055Z.db (24576 bytes)
```

- Writes to `backups/<name>-<UTC timestamp>.db`. Override the directory with
  `BACKUP_DIR=/path just db-backup`.
- `backups/` is git-ignored; snapshots are data, not source.
- The snapshot is verified with `integrity_check` before the command succeeds.

## Restore

```bash
just db-restore backups/app-20260621T150055Z.db
```

Restore is deliberately careful — it never clobbers a database silently:

1. It first runs `integrity_check` on the **backup** and refuses to proceed if the
   file is not a healthy SQLite database.
2. If a current database exists, it prompts for confirmation. Set `FORCE=1` to
   skip the prompt for automation: `FORCE=1 just db-restore <file>`.
3. Before overwriting, it snapshots the current database to
   `backups/<name>-pre-restore-<timestamp>.db`, so a restore is reversible.
4. It copies the snapshot into place, removes stale `-wal`/`-shm` sidecars so the
   restored file is the single source of truth, and verifies the result.

Stop the server before restoring — restoring under a running writer races the
live connection. A clean shutdown (the app handles SIGTERM and drains) checkpoints
WAL, so the database is consistent before you swap files.

## Integrity check

```bash
just db-check
# Database: /repo/data/app.db
# PRAGMA integrity_check: ok
#
# Applied migrations:
#   20260611000001  create items
#   20260613000001  create posts
#   20260621000001  add posts status check
```

- Exits non-zero if the check fails, so `just db-check` works as a CI gate or a
  cron health probe.
- `QUICK=1 just db-check` runs the faster, less exhaustive `quick_check` instead.
- It also reports the migrations recorded in sqlx's `_sqlx_migrations` table, so
  you can confirm a restored database is at the schema version you expect.
- `integrity_check` validates the database structure end to end — the standard
  SQLite health check. A structural check still cannot prove every byte of
  application data is what you intended, so rehearse a real restore (below).

## Docker

The image stores SQLite under `/data` and declares it a volume
([`Dockerfile`](../../Dockerfile)), so the database survives container
replacement. Mount a named volume or a host directory there:

```yaml
# compose.yaml
services:
  app:
    volumes:
      - app-data:/data        # named volume
      # - ./data:/data        # or a host bind mount
volumes:
  app-data:
```

The runtime image is minimal and has no `sqlite3`, so back up from outside the
container. Two options:

- **Host bind mount** — if `/data` is bind-mounted from the host (`./data:/data`),
  the host sees the same file: `DATABASE_URL="sqlite://data/app.db?mode=rwc" just db-backup`.
- **Named volume, no downtime** — run a throwaway container that has `sqlite3`,
  sharing the volume, and write the snapshot to a host directory:

  ```bash
  docker run --rm \
    -v app-data:/data \
    -v "$PWD/backups:/backups" \
    alpine:3 sh -c 'apk add --no-cache sqlite >/dev/null && \
      sqlite3 /data/app.db ".backup /backups/app-$(date -u +%Y%m%dT%H%M%SZ).db"'
  ```

To restore in Docker, stop the container, replace `/data/app.db` with your
snapshot and delete `/data/app.db-wal` and `/data/app.db-shm`, then start it
again.

## Desktop

The Tauri shell runs the server as a sidecar with its working directory set to
the OS application-data directory, so the database is `app.db` inside that
directory (for example, on macOS,
`~/Library/Application Support/<bundle id>/app.db`). Point `DATABASE_URL` at it to
snapshot a desktop install:

```bash
DATABASE_URL="sqlite://$HOME/Library/Application Support/com.example.app-starter/app.db" just db-backup
```

## Test your restore on a throwaway environment

A backup you have never restored is a hope, not a backup. Prove the round-trip
against a scratch copy, never your live database:

Run this from the repository root:

```bash
# 1. Snapshot the live database, then grab the newest backup file.
just db-backup
backup="$(ls -t backups/*.db | head -1)"

# 2. Restore it into a throwaway path — never your live database.
scratch=/tmp/restore-test && mkdir -p "$scratch"
FORCE=1 DATABASE_URL="sqlite://$scratch/app.db?mode=rwc" just db-restore "$backup"

# 3. Verify the throwaway copy is healthy and at the schema version you expect.
DATABASE_URL="sqlite://$scratch/app.db?mode=rwc" just db-check
```

If `db-check` reports `ok` and the migrations you expect, the snapshot is good.

## Scheduling

These are plain commands, so any scheduler works. A daily cron snapshot with a
30-day retention sweep:

```cron
0 3 * * *  cd /srv/app-starter && just db-backup && find backups -name '*.db' -mtime +30 -delete
```

Keep snapshots on different storage than the live database — a backup on the same
disk does not survive that disk failing.
