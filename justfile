set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Run the backend (serves the embedded UI when interface/dist exists)
dev:
    cargo run

# Run the Vite dev server with /api proxied to the backend
frontend-dev:
    cd interface && bun run dev

# Regenerate TypeScript types from the OpenAPI spec
typegen:
    cargo run --bin openapi_spec > /tmp/openapi.json
    cd interface && bunx openapi-typescript /tmp/openapi.json -o src/api/schema.d.ts

fmt:
    cargo fmt --all

# Fail if the committed TS types are stale relative to the OpenAPI spec
check-typegen: typegen
    git diff --exit-code -- interface/src/api/schema.d.ts

# Fast gate: fmt check + clippy -D warnings + frontend Biome + tsc. Run `just verify` for the full CI set.
lint:
    cargo fmt --all -- --check
    SKIP_FRONTEND_BUILD=1 cargo clippy --all-targets -- -D warnings
    cd interface && bunx biome check .
    cd interface && bunx tsc --noEmit

# Everything CI runs, locally: lint + backend tests + typegen drift + frontend build/test + cargo-deny.
verify: lint test check-typegen
    cd interface && bun run build && bun run test
    if command -v cargo-deny >/dev/null; then cargo deny check; else echo "note: cargo-deny not installed locally; it runs in CI"; fi

test:
    SKIP_FRONTEND_BUILD=1 cargo test

# Production build: frontend first, then the binary that embeds it
build:
    cd interface && bun install && bun run build
    SKIP_FRONTEND_BUILD=1 cargo build --release

docker-build:
    docker build -t app-starter .

# Tauri desktop app (needs platform prerequisites: https://tauri.app/start/prerequisites/)
desktop-dev:
    cd desktop && bun install && bun run dev

# Desktop app with full hot reload (backend via cargo-watch, UI via Vite). Needs cargo-watch.
desktop-dev-hot:
    #!/usr/bin/env bash
    # cargo-watch rebuilds the backend on Rust changes; the shell hot-reloads the
    # UI via Vite. Stop with Ctrl-C or by closing the window.
    set -euo pipefail
    if ! command -v cargo-watch >/dev/null; then
        echo "cargo-watch not found. Install it with: cargo install cargo-watch" >&2
        exit 1
    fi
    # Start the live-reloading backend first and keep it on :8080. The desktop
    # shell still auto-spawns its bundled sidecar, but it loses the port race to
    # this one and exits, so cargo-watch serves the window with backend reloads.
    cargo watch -x run &
    backend=$!
    trap 'kill "$backend" 2>/dev/null || true' EXIT
    echo "waiting for the backend on :8080 before launching the desktop shell…"
    until curl -sf http://127.0.0.1:8080/api/health >/dev/null 2>&1; do
        sleep 0.5
    done
    cd desktop && bun install && bun run dev

desktop-build:
    cd desktop && bun install && bun run build

# Point git at the bundled hooks (cargo fmt on commit)
hooks:
    git config core.hooksPath .githooks
