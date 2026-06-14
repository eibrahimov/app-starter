#!/usr/bin/env bash
# Read-only release preflight for App Starter. Run BEFORE `git tag`.
# Verifies the working tree is clean, Cargo.toml and Cargo.lock agree on the version, and
# the vX.Y.Z tag does not already exist -- catching the exact tag/version mismatch that
# .github/workflows/release.yml hard-fails on. Makes NO changes (no tag, no push).
# Exit code: 0 if ready to tag, 1 otherwise.

set -uo pipefail

FAILED=0
note() { printf '\n=== %s ===\n' "$1"; }
ok()   { printf 'PASS: %s\n' "$1"; }
warn() { printf 'WARN: %s\n' "$1"; }
bad()  { printf 'FAIL: %s\n' "$1"; FAILED=1; }

if ! ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  echo "FAIL: not in a git repo." >&2; exit 1
fi
cd "$ROOT" || exit 1
[ -f Cargo.toml ] || { echo "FAIL: no Cargo.toml at repo root." >&2; exit 1; }
echo "Repo root: $ROOT"

# --- version from Cargo.toml ------------------------------------------------
note "Version"
VER="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
if [ -z "$VER" ]; then bad "could not read version from Cargo.toml."; else ok "Cargo.toml version: $VER"; fi

# --- Cargo.lock agreement ---------------------------------------------------
note "Cargo.lock agreement"
if [ -f Cargo.lock ]; then
  LOCK_VER="$(awk '/^name = "app-starter"$/{getline; print}' Cargo.lock | head -1 | cut -d '"' -f2)"
  if [ -z "$LOCK_VER" ]; then
    warn "could not find app-starter version in Cargo.lock."
  elif [ "$LOCK_VER" = "$VER" ]; then
    ok "Cargo.lock matches ($LOCK_VER)."
  else
    bad "Cargo.lock has $LOCK_VER but Cargo.toml has $VER. Run 'cargo update -p app-starter' and commit Cargo.lock."
  fi
else
  warn "no Cargo.lock present."
fi

# --- clean working tree -----------------------------------------------------
note "Working tree"
if [ -n "$(git status --porcelain)" ]; then
  bad "working tree is dirty. Commit the release bump before tagging."
else
  ok "working tree clean."
fi

# --- tag availability -------------------------------------------------------
note "Tag availability"
if [ -n "$VER" ]; then
  if git rev-parse -q --verify "refs/tags/v$VER" >/dev/null; then
    bad "tag v$VER already exists. Bump the version or delete the stale tag."
  else
    ok "tag v$VER is available."
  fi
fi

# --- branch sanity ----------------------------------------------------------
note "Branch"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)"
if [ "$BRANCH" = "master" ] || [ "$BRANCH" = "main" ]; then
  ok "on $BRANCH."
else
  warn "on '$BRANCH', not master/main. Releases are normally tagged on the default branch."
fi

# --- summary ----------------------------------------------------------------
note "Summary"
if [ "$FAILED" -eq 0 ]; then
  echo "Preflight passed. Next: git tag v$VER && git push origin $BRANCH --tags (publishes to GHCR -- confirm intent)."
  exit 0
else
  echo "Preflight failed (see FAIL lines). Do not tag yet."
  exit 1
fi
