#!/usr/bin/env bash
# Validate a newly added resource against the template's deterministic gates.
#
# Mirrors the project's own commands (justfile) instead of re-implementing them:
#   1. typegen drift     -> `just check-typegen`  (regenerate types, fail if they
#                           differ from the committed interface/src/api/schema.d.ts)
#   2. backend tests     -> `just test`           (includes two guard tests for the
#                           silent three-place-registration footgun:
#                           openapi_spec_has_no_dangling_schema_refs catches a missing
#                           components(schemas(...)) entry, and
#                           routes_and_openapi_spec_are_in_parity catches a route<->spec
#                           /api/v1 path mismatch -- a handler routed but absent from
#                           paths(...), or vice versa)
#
# Prefers `just` (the documented dev UX); falls back to the raw commands when `just`
# is not installed. Run from anywhere inside the repo.
#
# Exit code: 0 if all gates pass, 1 otherwise. This is a fast pre-handoff check;
# run `just verify` for the complete CI set (lint + frontend build/test + cargo-deny).

set -uo pipefail

GUARD_TEST="openapi_spec_has_no_dangling_schema_refs"
PARITY_TEST="routes_and_openapi_spec_are_in_parity"
FAILED=0

note() { printf '\n=== %s ===\n' "$1"; }
ok()   { printf 'PASS: %s\n' "$1"; }
bad()  { printf 'FAIL: %s\n' "$1"; FAILED=1; }

# --- locate repo root -------------------------------------------------------
if ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  :
else
  ROOT="$PWD"
  while [ "$ROOT" != "/" ] && [ ! -f "$ROOT/Cargo.toml" ]; do ROOT="$(dirname "$ROOT")"; done
fi
if [ ! -f "$ROOT/Cargo.toml" ] || [ ! -f "$ROOT/justfile" ]; then
  echo "FAIL: could not find the app-starter repo root (need Cargo.toml + justfile)." >&2
  exit 1
fi
cd "$ROOT" || exit 1
echo "Repo root: $ROOT"

have() { command -v "$1" >/dev/null 2>&1; }
HAVE_JUST=0; have just && HAVE_JUST=1

if ! have cargo; then
  echo "FAIL: cargo not found on PATH; cannot validate." >&2
  exit 1
fi

# --- gate 1: typegen drift --------------------------------------------------
note "Gate 1/2: typegen drift (committed TS types match the OpenAPI spec)"
if [ "$HAVE_JUST" -eq 1 ]; then
  if just check-typegen; then ok "typegen up to date"; else
    bad "typegen drift: interface/src/api/schema.d.ts is stale. Run 'just typegen' and commit it (recipe step 7)."
  fi
elif have bunx; then
  if [ ! -d interface/node_modules ]; then
    echo "note: interface/node_modules missing; running 'bun install' first."
    ( cd interface && bun install ) || bad "bun install failed"
  fi
  cargo run --bin openapi_spec > /tmp/openapi.json \
    && ( cd interface && bunx openapi-typescript /tmp/openapi.json -o src/api/schema.d.ts )
  if git diff --exit-code -- interface/src/api/schema.d.ts; then ok "typegen up to date"; else
    bad "typegen drift: schema.d.ts changed after regeneration. Commit the regenerated file (recipe step 7)."
  fi
else
  echo "SKIP: neither 'just' nor 'bunx' available; cannot check typegen drift here (CI still enforces it)."
fi

# --- gate 2: backend tests (incl. the dangling-ref guard) -------------------
note "Gate 2/2: backend tests, including the $GUARD_TEST guard"
TEST_LOG="$(mktemp)"
if [ "$HAVE_JUST" -eq 1 ]; then
  just test 2>&1 | tee "$TEST_LOG"; RC=${PIPESTATUS[0]}
else
  SKIP_FRONTEND_BUILD=1 cargo test 2>&1 | tee "$TEST_LOG"; RC=${PIPESTATUS[0]}
fi
if [ "$RC" -eq 0 ]; then
  ok "backend tests passed"
  if grep -q "$GUARD_TEST" "$TEST_LOG"; then
    ok "OpenAPI dangling-\$ref guard ($GUARD_TEST) ran"
  else
    echo "note: did not see '$GUARD_TEST' in output; confirm it still exists in tests/api.rs."
  fi
  if grep -q "$PARITY_TEST" "$TEST_LOG"; then
    ok "route<->spec parity guard ($PARITY_TEST) ran"
  else
    echo "note: did not see '$PARITY_TEST' in output; confirm it still exists in tests/api.rs."
  fi
else
  bad "backend tests failed. If $GUARD_TEST failed, a handler's ToSchema type is missing from components(schemas(...)) in src/api.rs (recipe step 5). If $PARITY_TEST failed, a handler is registered in only one of paths(...) / .route(...) -- wire it in both (recipe step 5)."
fi
rm -f "$TEST_LOG"

# --- summary ----------------------------------------------------------------
note "Summary"
if [ "$FAILED" -eq 0 ]; then
  echo "All resource gates passed. Run 'just verify' for the full CI set before handoff."
  exit 0
else
  echo "One or more gates failed (see FAIL lines above). Fix before handoff."
  exit 1
fi
