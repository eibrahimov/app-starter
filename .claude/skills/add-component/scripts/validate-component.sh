#!/usr/bin/env bash
# Validate newly added frontend components/sections/hooks against the template's
# deterministic gates.
#
# Mirrors the project's own commands instead of re-implementing them:
#   1. Biome   -> `bunx biome check .`   (format + lint)
#   2. tsc     -> `bunx tsc --noEmit`     (types, incl. the openapi-fetch generics)
#   3. Vitest  -> `bun run test`          (existing + new component tests)
# Plus two fast grep guards over the new files for the frontend footguns:
#   - `@/` path-alias imports (forbidden — use relative imports)
#   - `<button` without an explicit `type=` (use the Button primitive instead)
#
# Run from anywhere inside the repo. Exit 0 if all gates pass, 1 otherwise. This
# is a fast pre-handoff check; run `just verify` for the complete CI set.

set -uo pipefail

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
if [ ! -f "$ROOT/Cargo.toml" ] || [ ! -d "$ROOT/interface" ]; then
  echo "FAIL: could not find the app-starter repo root (need Cargo.toml + interface/)." >&2
  exit 1
fi
cd "$ROOT/interface" || exit 1
echo "Interface root: $ROOT/interface"

have() { command -v "$1" >/dev/null 2>&1; }
if ! have bunx; then
  echo "FAIL: bunx not found on PATH; cannot validate the frontend." >&2
  exit 1
fi
if [ ! -d node_modules ]; then
  echo "note: interface/node_modules missing; running 'bun install' first."
  bun install || bad "bun install failed"
fi

# --- gate 1: Biome ----------------------------------------------------------
note "Gate 1/3: Biome (format + lint)"
if bunx biome check .; then ok "Biome clean"; else
  bad "Biome found issues. Run 'bunx biome check --write .' to autofix formatting; fix lint errors by hand."
fi

# --- gate 2: tsc ------------------------------------------------------------
note "Gate 2/3: TypeScript (tsc --noEmit)"
if bunx tsc --noEmit; then ok "types check"; else
  bad "tsc failed. Component props or the openapi-fetch generics do not type-check."
fi

# --- gate 3: Vitest ---------------------------------------------------------
note "Gate 3/3: Vitest"
if bun run test; then ok "tests passed"; else
  bad "Vitest failed. A component test (or a refactored page test) is red."
fi

# --- guard: forbidden patterns in source -----------------------------------
note "Guard: frontend footguns in src/"
if grep -rniE 'from\s+"\@/' src >/dev/null 2>&1; then
  bad "Found '@/' path-alias imports in src/ — use relative imports (invariant 2)."
else
  ok "no '@/' path-alias imports"
fi
# Flag <button ...> elements that have no type= attribute (prefer the Button primitive).
# Heuristic only: this is a single-line grep. It reliably catches a one-line
# `<button>` that is missing type=, but it cannot correlate a multi-line opening
# tag — if `type=` sits on a later line, the bare `<button` line survives the
# `type=` filter and is flagged as a FALSE POSITIVE. So treat a hit as a prompt
# to look, not a hard failure. Biome's a11y lint and code review are the real
# gates; this is a fast smoke check, not an authoritative one.
BUTTONS_NO_TYPE="$(grep -rnE '<button(\s|>)' src 2>/dev/null | grep -vE 'type=' || true)"
if [ -n "$BUTTONS_NO_TYPE" ]; then
  bad "Found <button> without an explicit type= (use the Button primitive, invariant 4):"
  printf '%s\n' "$BUTTONS_NO_TYPE"
else
  ok "every <button> has an explicit type="
fi

# --- guard: hard-coded palette (prefer semantic tokens) ---------------------
note "Guard: hard-coded palette colours in src/"
PALETTE="$(grep -rnE '(zinc|slate|gray|neutral|stone|emerald|amber)-[0-9]{2,3}' src/components src/pages 2>/dev/null || true)"
if [ -n "$PALETTE" ]; then
  echo "note: hard-coded palette colours found — prefer semantic token utilities"
  echo "      (bg-card, text-muted-foreground, text-destructive, …) so light/dark theming works:"
  printf '%s\n' "$PALETTE"
else
  ok "no hard-coded palette colours (semantic tokens only)"
fi

# --- summary ----------------------------------------------------------------
note "Summary"
if [ "$FAILED" -eq 0 ]; then
  echo "All component gates passed. Run 'just verify' for the full CI set before handoff."
  exit 0
else
  echo "One or more gates failed (see FAIL lines above). Fix before handoff."
  exit 1
fi
