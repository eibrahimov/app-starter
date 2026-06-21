#!/usr/bin/env bash
# Smoke test for scripts/db.sh: the backup -> mutate -> restore -> check round-trip
# plus the guard rails (reject :memory:/non-SQLite URLs, refuse a corrupt backup,
# leave the live DB intact on a bad restore), all against a scratch WAL database
# in a temp dir -- never your real data. Run via `just db-selftest` or
# `bash scripts/db.test.sh`. Exits 0 when every assertion holds, 1 otherwise.
#
# Needs only bash + sqlite3 -- the same prerequisite as db.sh, no test harness.

set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_SH="$HERE/db.sh"

PASS=0
FAIL=0
ok()   { printf 'ok   - %s\n' "$1"; PASS=$((PASS + 1)); }
fail() { printf 'FAIL - %s\n' "$1" >&2; FAIL=$((FAIL + 1)); }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

DB="$TMP/app.db"
export BACKUP_DIR="$TMP/backups"

# Seed a WAL-mode database with three rows, the way the server leaves it.
seed_db() {
  rm -f "$DB" "$DB-wal" "$DB-shm"
  sqlite3 "$DB" \
    "PRAGMA journal_mode=WAL; CREATE TABLE t(id INTEGER PRIMARY KEY); INSERT INTO t VALUES (1),(2),(3);" \
    >/dev/null
}
rows() { sqlite3 "$DB" "SELECT count(*) FROM t;" 2>/dev/null; }

# Run db.sh with an absolute DATABASE_URL in the scratch dir, so the repo's
# ROOT/.env and real database are never consulted.
run() { DATABASE_URL="sqlite://$DB?mode=rwc" bash "$DB_SH" "$@"; }

# 1) check on a healthy database succeeds.
seed_db
if run check >/dev/null 2>&1; then ok "check: healthy DB exits 0"; else fail "check: healthy DB should exit 0"; fi

# 2) backup writes a verified snapshot.
if run backup >/dev/null 2>&1; then ok "backup: exits 0"; else fail "backup: should exit 0"; fi
snap_file="$(ls -t "$BACKUP_DIR"/*.db 2>/dev/null | head -1 || true)"
if [ -n "$snap_file" ] && [ -f "$snap_file" ]; then ok "backup: snapshot file written"; else fail "backup: no snapshot file"; fi

# 3) restore rolls a mutated database back to the snapshot.
sqlite3 "$DB" "INSERT INTO t VALUES (4),(5);" >/dev/null
[ "$(rows)" = "5" ] || fail "setup: expected 5 rows after mutation"
if FORCE=1 run restore "$snap_file" >/dev/null 2>&1; then ok "restore: exits 0"; else fail "restore: should exit 0"; fi
if [ "$(rows)" = "3" ]; then ok "restore: rolled back to 3 rows"; else fail "restore: expected 3 rows, got $(rows)"; fi
if [ ! -e "$DB-wal" ] && [ ! -e "$DB-shm" ]; then ok "restore: stale -wal/-shm cleared"; else fail "restore: -wal/-shm not cleared"; fi
if ls "$BACKUP_DIR"/*pre-restore*.db >/dev/null 2>&1; then ok "restore: pre-restore snapshot saved"; else fail "restore: no pre-restore snapshot"; fi

# 4) :memory: and non-SQLite URLs are refused with a non-zero exit.
if DATABASE_URL="sqlite::memory:" bash "$DB_SH" backup >/dev/null 2>&1; then fail ":memory: backup should be refused"; else ok ":memory: URL refused"; fi
if DATABASE_URL="postgres://u:p@h/db" bash "$DB_SH" check >/dev/null 2>&1; then fail "non-SQLite check should be refused"; else ok "non-SQLite URL refused"; fi

# 5) a corrupt backup is refused and the live database is left intact.
seed_db
corrupt="$TMP/corrupt.db"
printf 'not a database' > "$corrupt"
if FORCE=1 run restore "$corrupt" >/dev/null 2>&1; then fail "restore of a corrupt backup should be refused"; else ok "restore: corrupt backup refused"; fi
if [ "$(rows)" = "3" ]; then ok "restore: live DB intact after a refused restore"; else fail "restore: live DB altered by a refused restore"; fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then exit 0; else exit 1; fi
