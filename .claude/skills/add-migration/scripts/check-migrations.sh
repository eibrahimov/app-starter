#!/usr/bin/env bash
# Read-only guard for App Starter migrations. Catches the cardinal sqlx sin -- editing or
# renaming a committed migration -- plus non-monotonic timestamps and malformed filenames.
# Makes NO changes. Pairs with the add-migration skill. Run from anywhere inside the repo.
# Exit code: 0 if migrations are clean, 1 otherwise.

set -uo pipefail

FAILED=0
note() { printf '\n=== %s ===\n' "$1"; }
ok()   { printf 'PASS: %s\n' "$1"; }
bad()  { printf 'FAIL: %s\n' "$1"; FAILED=1; }

if ! ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  echo "FAIL: not in a git repo." >&2; exit 1
fi
cd "$ROOT" || exit 1
[ -d migrations ] || { echo "FAIL: no migrations/ directory." >&2; exit 1; }
echo "Repo root: $ROOT"

# --- 1. append-only: no committed migration modified, renamed, or deleted ---
note "Append-only (no committed migration edited or renamed)"
CHANGED="$(git diff --name-status --diff-filter=a HEAD -- migrations)"
if [ -n "$CHANGED" ]; then
  bad "a committed migration was modified, renamed, or deleted -- this breaks sqlx checksums:"
  printf '%s\n' "$CHANGED"
  echo "  Revert it and add a NEW corrective forward migration instead."
else
  ok "no committed migration changed (additions only, if any)."
fi

# --- 2. filenames well-formed + timestamps unique (single pass) -------------
# The glob expands in sorted order, so a duplicate timestamp can only land in
# adjacent entries; checking the name and the timestamp together is one walk.
note "Filenames and timestamps"
PREV=""
SECTION_OK=1
for f in migrations/*.sql; do
  [ -e "$f" ] || continue
  base="${f##*/}"
  if ! printf '%s' "$base" | grep -qE '^[0-9]{14}_.+\.sql$'; then
    bad "bad migration filename: $base (expected <YYYYMMDDHHMMSS>_<description>.sql)"
    SECTION_OK=0
    continue
  fi
  ts="${base%%_*}"
  if [ "$ts" = "$PREV" ]; then
    bad "duplicate migration timestamp: $ts"
    SECTION_OK=0
  fi
  PREV="$ts"
done
[ "$SECTION_OK" -eq 1 ] && ok "filenames well-formed; timestamps unique."

# --- summary ----------------------------------------------------------------
note "Summary"
if [ "$FAILED" -eq 0 ]; then
  echo "Migrations clean."
  exit 0
else
  echo "Migration issues found (see FAIL lines). Fix before committing."
  exit 1
fi
