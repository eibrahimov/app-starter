#!/usr/bin/env bash
# Read-only production-readiness audit for App Starter.
# Surfaces the deliberate decisions in docs/production-readiness.md as PASS/WARN/FAIL.
# Makes NO changes. FAIL = blocker (e.g. a committed secret); WARN = a decision you must
# make before public exposure (e.g. permissive CORS). Pairs with the harden-for-production
# skill. Run from anywhere inside the repo.
#
# Exit code: 1 if any FAIL, else 0 (WARNs do not fail -- they are decisions, not bugs).

set -uo pipefail

FAILED=0
WARNED=0
note() { printf '\n=== %s ===\n' "$1"; }
ok()   { printf 'PASS: %s\n' "$1"; }
warn() { printf 'WARN: %s\n' "$1"; WARNED=$((WARNED + 1)); }
bad()  { printf 'FAIL: %s\n' "$1"; FAILED=1; }

# --- locate repo root (git required: the secrets check uses git ls-files) ----
if ! ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  echo "FAIL: not in a git repo (production-audit uses git to check tracked files)." >&2; exit 1
fi
cd "$ROOT" || exit 1
[ -f Cargo.toml ] || { echo "FAIL: not inside the app-starter repo (no Cargo.toml)." >&2; exit 1; }
echo "Repo root: $ROOT"

# --- secrets ----------------------------------------------------------------
note "Secrets"
if git ls-files --error-unmatch .env >/dev/null 2>&1; then
  bad ".env is tracked by git. Remove it from version control; keep secrets out of the repo."
else
  ok ".env is not tracked."
fi
if [ -f .env.example ]; then
  ok ".env.example present (keep it complete but non-secret)."
else
  warn ".env.example missing; document required env vars for the generated app."
fi

# --- public exposure / CORS -------------------------------------------------
note "Public exposure / CORS"
if grep -rqs "CorsLayer::permissive()" src/; then
  warn "CorsLayer::permissive() is active (needed for the Tauri sidecar). Restrict origins or explicitly accept it before public exposure (docs/production-readiness.md -> Public exposure and CORS)."
else
  ok "Permissive CORS not found; confirm origins are restricted as intended."
fi

# --- desktop sidecar --------------------------------------------------------
note "Desktop sidecar"
TAURI_CONF="desktop/src-tauri/tauri.conf.json"
if [ -f "$TAURI_CONF" ]; then
  if grep -qs "com.example" "$TAURI_CONF"; then
    warn "Tauri bundle identifier still uses com.example.* in $TAURI_CONF; change it to your reverse domain before shipping desktop builds."
  else
    ok "Tauri bundle identifier customized."
  fi
else
  ok "No desktop/Tauri config present (skipping)."
fi

# --- release versioning -----------------------------------------------------
note "Release versioning"
VER="$(grep -m1 '^version' Cargo.toml | cut -d '"' -f2)"
if [ -n "$VER" ]; then
  ok "Cargo.toml version: $VER (the release workflow requires tag v$VER to match)."
else
  warn "Could not read version from Cargo.toml."
fi

# --- summary ----------------------------------------------------------------
note "Summary"
echo "FAIL: $FAILED block(s), WARN: $WARNED decision(s) to make."
echo "Full checklist: docs/production-readiness.md"
if [ "$FAILED" -ne 0 ]; then
  echo "Resolve FAIL lines before exposing publicly."
  exit 1
fi
exit 0
