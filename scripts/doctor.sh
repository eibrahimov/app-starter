#!/usr/bin/env bash
# Read-only environment + build-state preflight for App Starter.
# Run via `just doctor` (or `bash scripts/doctor.sh`) on a fresh clone to surface
# a missing tool or build step BEFORE it fails deep inside a cargo/bun build.
# Makes NO changes. Exit 0 when ready to run; 1 when a hard prerequisite is missing.
#
# The fix-command strings below mirror the README "Troubleshooting" section --
# keep the two in sync when either side changes.
# The pinned Rust version is read from rust-toolchain.toml at runtime; do NOT
# hardcode it here, or it becomes a second source of truth that drifts when the
# pin is bumped.

set -uo pipefail

FAILED=0
note() { printf '\n=== %s ===\n' "$1"; }
ok()   { printf 'PASS: %s\n' "$1"; }
warn() { printf 'WARN: %s\n' "$1"; }
bad()  { printf 'FAIL: %s\n' "$1"; FAILED=1; }

# Anchor to the repo root so the dir/file checks work from any CWD.
if ! ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
cd "$ROOT" || exit 1
echo "Repo root: $ROOT"

# --- Rust toolchain ---------------------------------------------------------
note "Rust toolchain"
if command -v rustc >/dev/null; then
  RUSTC_VER="$(rustc --version | awk '{print $2}')"
  ok "rustc present ($RUSTC_VER)."
  if [ -f rust-toolchain.toml ]; then
    PIN="$(grep -m1 '^channel' rust-toolchain.toml | cut -d '"' -f2)"
    if [ -z "$PIN" ]; then
      warn "could not read the channel from rust-toolchain.toml."
    elif ! printf '%s' "$PIN" | grep -qE '^[0-9]+\.[0-9]+'; then
      # Non-version channel (e.g. stable/nightly/dated nightly): rustup resolves
      # it, so an exact rustc compare would misfire -- just report it.
      ok "rust-toolchain.toml channel is '$PIN' (rustup resolves it automatically)."
    elif [ "$PIN" = "$RUSTC_VER" ]; then
      ok "matches the rust-toolchain.toml pin ($PIN)."
    else
      warn "rustc is $RUSTC_VER but rust-toolchain.toml pins $PIN; rustup fetches the pin on first build, or install now: rustup toolchain install $PIN"
    fi
  fi
else
  bad "rustc not found. Install Rust from https://rustup.rs, then re-run 'just doctor'."
fi

if command -v cargo >/dev/null; then
  ok "cargo present."
else
  bad "cargo not found. Install Rust from https://rustup.rs."
fi

# --- Frontend toolchain (Bun) ----------------------------------------------
note "Frontend toolchain"
if command -v bun >/dev/null; then
  ok "bun present ($(bun --version))."
else
  bad "bun not found. Install Bun from https://bun.sh, then: cd interface && bun install"
fi

# --- Frontend install + build state ----------------------------------------
note "Frontend build state"
if [ -d interface/node_modules ]; then
  ok "interface/node_modules present."
  # Install drift is the silent footgun: node_modules exists but is stale or
  # incomplete, so the dev server only fails to resolve a dependency at runtime
  # (e.g. a dep in package.json/bun.lock that was never physically installed).
  # Two independent, read-only checks (no install, no lockfile write):
  if command -v bun >/dev/null; then
    # (a) bun.lock must satisfy package.json: --frozen-lockfile exits nonzero
    #     when they disagree (a dep added/bumped without updating the lockfile).
    if (cd interface && bun install --frozen-lockfile --dry-run) >/dev/null 2>&1; then
      ok "interface/bun.lock is in sync with package.json (frozen-lockfile)."
    else
      warn "interface/bun.lock is out of sync with package.json. Run: cd interface && bun install"
    fi
    # (b) --frozen-lockfile still passes when a locked package is merely absent
    #     from node_modules, so verify each declared dep is physically present.
    MISSING_DEPS="$(cd interface && bun -e 'const fs=require("fs");const p=require("./package.json");const n=[...Object.keys(p.dependencies||{}),...Object.keys(p.devDependencies||{})];process.stdout.write(n.filter(x=>!fs.existsSync("node_modules/"+x)).join(" "))' 2>/dev/null)"
    if [ -n "$MISSING_DEPS" ]; then
      warn "interface deps declared but not installed: $MISSING_DEPS. Run: cd interface && bun install"
    else
      ok "all declared interface deps are present in node_modules."
    fi
  fi
else
  warn "interface deps not installed. Run: cd interface && bun install"
fi
if [ -d interface/dist ]; then
  ok "interface/dist present (the binary can embed and serve the UI)."
else
  warn "interface/dist missing -- 'cargo run' will serve the 'frontend not built' page. Run: cd interface && bun install && bun run build, then cargo run"
fi

# --- Optional tooling (advisory only) --------------------------------------
note "Optional tooling"
if command -v just >/dev/null; then
  ok "just present."
else
  warn "just not found (optional). Install from https://github.com/casey/just, or run the justfile recipes directly."
fi
if command -v cargo-deny >/dev/null; then
  ok "cargo-deny present."
else
  warn "cargo-deny not installed (optional; 'just verify' and CI run the license/advisory check). Install with: cargo install cargo-deny"
fi
if [ -d "$HOME/.cache/ms-playwright" ] || [ -d "$HOME/Library/Caches/ms-playwright" ]; then
  ok "Playwright browsers present (for 'just a11y')."
else
  warn "Playwright chromium not installed (only needed for 'just a11y'). One-time: cd interface && bunx playwright install chromium"
fi

# --- summary ----------------------------------------------------------------
note "Summary"
if [ "$FAILED" -eq 0 ]; then
  echo "Environment looks good. Next: 'cargo run' (backend on :8080) and 'just frontend-dev' for the UI."
  exit 0
else
  echo "Missing a hard prerequisite (see FAIL lines above). Fix those, then re-run 'just doctor'."
  exit 1
fi
