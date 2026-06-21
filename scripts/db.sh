#!/usr/bin/env bash
# SQLite backup / restore / integrity helper for App Starter.
# Run via the just recipes (`just db-backup`, `just db-restore <file>`,
# `just db-check`) or directly: `bash scripts/db.sh {backup|restore <file>|check}`.
#
# Operates on the DB FILE only -- no schema changes, no migrations. The target
# file is derived from DATABASE_URL the same way src/db.rs derives it (strip the
# sqlite:// scheme, drop the ?query, reject :memory:), so this tooling acts on
# the same database the server uses, for the dev DB and any fork's configured
# path alike.
#
# Why a dedicated helper and not `cp`: sqlx opens SQLite in WAL mode, so the
# live database is split across <db>, <db>-wal, and <db>-shm. A raw `cp` of the
# .db file alone can capture a torn page or miss the committed -wal tail. The
# online backup API (`.backup`) used here takes a consistent snapshot of a live
# single-writer database into one self-contained file.

set -uo pipefail

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# Anchor to the repo root so a relative DATABASE_URL path (the default
# `data/app.db`) resolves against it -- the server's working directory in the
# standard `cargo run` and Docker setups. For other layouts (e.g. a desktop
# install, whose database lives in the OS app-data dir) pass an absolute
# DATABASE_URL; see docs/recipes/backup-restore.md.
if ! ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

require_sqlite3() {
  command -v sqlite3 >/dev/null 2>&1 || die "sqlite3 not found. Install it (macOS: ships with the OS, or 'brew install sqlite'; Debian/Ubuntu: 'apt-get install sqlite3')."
}

# Echo the effective DATABASE_URL: the environment wins, then the DATABASE_URL
# line in .env (the same file dotenvy loads at startup), then the built-in
# default from src/main.rs.
resolve_database_url() {
  if [ -n "${DATABASE_URL:-}" ]; then
    printf '%s' "$DATABASE_URL"
    return
  fi
  if [ -f "$ROOT/.env" ]; then
    local line
    line="$(grep -E '^[[:space:]]*DATABASE_URL=' "$ROOT/.env" | tail -n1 || true)"
    if [ -n "$line" ]; then
      line="${line#*=}"
      line="${line%\"}"; line="${line#\"}"
      line="${line%\'}"; line="${line#\'}"
      printf '%s' "$line"
      return
    fi
  fi
  printf '%s' 'sqlite://data/app.db?mode=rwc'
}

# Resolve the absolute SQLite file behind DATABASE_URL into the global DBFILE,
# mirroring src/db.rs (strip the sqlite scheme, drop the ?query, reject
# :memory:). Dies with a specific message for a non-SQLite URL or a URL with no
# file. Called directly -- not inside `$(...)` -- so die exits with one error.
DBFILE=""
resolve_db_file() {
  local url path
  url="$(resolve_database_url)"
  case "$url" in
    sqlite://*) path="${url#sqlite://}" ;;  # sqlite:///abs -> /abs ; sqlite://rel -> rel
    sqlite:*)   path="${url#sqlite:}" ;;     # sqlite:rel (no authority)
    *) die "DATABASE_URL is not a SQLite URL: '$url'. This tooling backs up sqlite:// databases only." ;;
  esac
  path="${path%%\?*}"                        # drop ?mode=rwc and friends
  case "$path" in
    "" | ":memory:") die "DATABASE_URL has no database file (':memory:' or empty); backup, restore, and check need a file-backed SQLite database." ;;
  esac
  case "$path" in
    /*) : ;;                # already absolute
    *)  path="$ROOT/$path" ;;
  esac
  DBFILE="$path"
}

# snapshot <src-db> <dest-file>: consistent online backup of a live DB into a new
# file. The destination is passed as a double-quoted .backup argument with
# backslash/double-quote escaping, so paths containing spaces or apostrophes work.
snapshot() {
  local esc="$2"
  esc="${esc//\\/\\\\}"   # backslash -> \\
  esc="${esc//\"/\\\"}"   # double quote -> \"
  sqlite3 "$1" ".backup \"$esc\"" || die "backup failed (sqlite3 .backup): $1 -> $2"
}

# integrity <db>: echo the PRAGMA integrity_check result ('ok' when healthy).
integrity() {
  sqlite3 "$1" 'PRAGMA integrity_check;' 2>/dev/null
}

backup() {
  local dest stamp dir stem
  resolve_db_file
  [ -f "$DBFILE" ] || die "database file not found: $DBFILE (run the app once to create it, or set DATABASE_URL)."
  require_sqlite3

  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  dir="${BACKUP_DIR:-$ROOT/backups}"
  mkdir -p "$dir" || die "could not create backup directory: $dir"
  stem="$(basename "$DBFILE")"; stem="${stem%.db}"
  dest="$dir/${stem}-${stamp}.db"

  snapshot "$DBFILE" "$dest"
  # Verify the snapshot opens and is structurally sound before claiming success.
  local res; res="$(integrity "$dest")"
  [ "$res" = "ok" ] || die "wrote $dest but its integrity_check failed: ${res:-unreadable}"
  printf 'Backed up %s -> %s (%s bytes)\n' "$DBFILE" "$dest" "$(wc -c < "$dest" | tr -d ' ')"
}

restore() {
  local file="${1:-}"
  [ -n "$file" ] || die "usage: just db-restore <backup-file>"
  [ -f "$file" ] || die "backup file not found: $file"
  require_sqlite3

  # Validate the backup BEFORE touching the live database.
  local res; res="$(integrity "$file")"
  [ "$res" = "ok" ] || die "refusing to restore: '$file' is not a healthy SQLite database (integrity_check: ${res:-unreadable})."

  resolve_db_file
  local dbfile="$DBFILE"

  # Guard: never clobber an existing database silently. Confirm interactively,
  # or set FORCE=1 for non-interactive/CI restores.
  if [ -f "$dbfile" ] && [ "${FORCE:-0}" != "1" ]; then
    printf 'About to overwrite %s\n            with %s\n' "$dbfile" "$file"
    printf 'Stop the server first. The current database is snapshotted before it is replaced.\n'
    printf 'Continue? [y/N] '
    local reply; read -r reply || reply=""
    case "$reply" in
      y|Y|yes|YES) ;;
      *) die "restore aborted (no changes made)." ;;
    esac
  fi

  # Reversible: snapshot the current database (if any) before replacing it.
  local safety=""
  if [ -f "$dbfile" ]; then
    local dir stem stamp
    dir="${BACKUP_DIR:-$ROOT/backups}"
    mkdir -p "$dir" || die "could not create backup directory: $dir"
    stem="$(basename "$dbfile")"; stem="${stem%.db}"
    stamp="$(date -u +%Y%m%dT%H%M%SZ)"
    safety="$dir/${stem}-pre-restore-${stamp}.db"
    snapshot "$dbfile" "$safety"
    printf 'Saved current database to %s\n' "$safety"
  fi

  # Atomic swap: copy to a temp file in the target directory, verify it, then
  # rename into place. The live database is only ever replaced by a complete,
  # verified file; a failed copy or a bad temp leaves the original untouched.
  mkdir -p "$(dirname "$dbfile")" || die "could not create database directory"
  local tmp="${dbfile}.restore.tmp"
  rm -f "$tmp"
  cp "$file" "$tmp" || die "copy failed: $file -> $tmp; live database left untouched${safety:+, pre-restore snapshot at $safety}."
  res="$(integrity "$tmp")"
  if [ "$res" != "ok" ]; then
    rm -f "$tmp"
    die "the copy of $file failed integrity_check (${res:-unreadable}); live database left untouched${safety:+, pre-restore snapshot at $safety}."
  fi
  mv -f "$tmp" "$dbfile" || die "could not move restored file into place: $tmp -> $dbfile"
  # Drop stale WAL/SHM sidecars so the restored file is the single source of truth.
  rm -f "${dbfile}-wal" "${dbfile}-shm"
  printf 'Restored %s -> %s\n' "$file" "$dbfile"
}

check() {
  resolve_db_file
  local dbfile="$DBFILE"
  [ -f "$dbfile" ] || die "database file not found: $dbfile (run the app once to create it, or set DATABASE_URL)."
  require_sqlite3

  printf 'Database: %s\n' "$dbfile"

  # Full check by default; QUICK=1 runs the faster, less exhaustive quick_check.
  local pragma="integrity_check"
  [ "${QUICK:-0}" = "1" ] && pragma="quick_check"
  local res; res="$(sqlite3 "$dbfile" "PRAGMA $pragma;" 2>&1)"
  if [ "$res" = "ok" ]; then
    printf 'PRAGMA %s: ok\n' "$pragma"
  else
    printf 'PRAGMA %s FAILED:\n%s\n' "$pragma" "$res" >&2
    exit 1
  fi

  # Report applied migrations from sqlx's bookkeeping table (absent until the
  # app has run migrations at least once).
  if [ "$(sqlite3 "$dbfile" "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations';" 2>/dev/null)" = "1" ]; then
    printf '\nApplied migrations:\n'
    sqlite3 -batch "$dbfile" "SELECT version || '  ' || description FROM _sqlx_migrations ORDER BY version;" | sed 's/^/  /'
  else
    printf '\nNo _sqlx_migrations table yet (database has not been migrated).\n'
  fi
}

case "${1:-}" in
  backup)  shift; backup ;;
  restore) shift; restore "${1:-}" ;;
  check)   shift; check ;;
  *)       die "usage: bash scripts/db.sh {backup|restore <file>|check}" ;;
esac
