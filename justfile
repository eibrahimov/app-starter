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

desktop-build:
    cd desktop && bun install && bun run build

# Point git at the bundled hooks (cargo fmt on commit)
hooks:
    git config core.hooksPath .githooks
