#!/usr/bin/env bash
# Release-profile plugin-registration smoke check (docs/plugin-framework.md §6,
# [review M1]). The silent-plugin-loss risk is a release `lto`+`strip` phenomenon
# (the shipping profile in Cargo.toml), so a debug-only assertion would miss it.
# This builds the RELEASE binary, boots it, and asserts every expected plugin's
# routes are actually served at /api/v1/<name> in /api/openapi.json.
#
# Run locally via `just release-smoke`; CI runs it on every push/PR.
set -euo pipefail

# Plugins that must be registered in a shipped build. Keep in sync with
# src/plugins.rs (the generated registry).
EXPECTED=("todo" "blog")

PORT="${PORT:-8091}"
DB="$(mktemp -u).db"

cleanup() {
    [ -n "${SRV:-}" ] && kill "$SRV" 2>/dev/null || true
    rm -f "${DB}" "${DB}-wal" "${DB}-shm"
}
trap cleanup EXIT

echo "Building release binary (lto+strip)..."
SKIP_FRONTEND_BUILD=1 cargo build --release --bin app-starter

echo "Booting release binary on port ${PORT}..."
PORT="${PORT}" DATABASE_URL="sqlite://${DB}?mode=rwc" ./target/release/app-starter &
SRV=$!

# Fetch the served spec, retrying until the server is up (curl handles the wait
# so the script never foreground-sleeps).
# `-fs` (not `-fsS`) so the expected connection-refused retries before boot stay
# quiet; `set -e` still fails the script if curl exhausts its retries.
spec=$(curl -fs --retry-connrefused --retry 60 --retry-delay 1 \
    "http://127.0.0.1:${PORT}/api/openapi.json")

fail=0
for name in "${EXPECTED[@]}"; do
    # Match the plugin's base path key OR any subpath under it ("/api/v1/<name>"
    # or "/api/v1/<name>/..."), so a plugin that only exposes subpaths (e.g.
    # /api/v1/<name>/{id}) still counts as registered. The trailing `(/|")` keeps
    # `<name>` from matching a longer sibling name (e.g. `blog` vs `blogarchive`).
    if printf '%s' "${spec}" | grep -qE "\"/api/v1/${name}(/|\")"; then
        echo "ok: plugin '${name}' registered (/api/v1/${name} served)"
    else
        echo "ERROR: plugin '${name}' is missing from the release spec (/api/v1/${name})" >&2
        fail=1
    fi
done

if [ "${fail}" -ne 0 ]; then
    echo "FAILED: one or more expected plugins were not registered in the release build." >&2
    exit 1
fi
echo "All expected plugins are registered in the release build."
