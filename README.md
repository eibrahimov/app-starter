# App Starter

Full-stack Rust starter. One binary serves the API and the UI, typed end to end via OpenAPI.

**Stack:** axum + SQLite (sqlx) on the backend. React 19 + Vite + Tailwind 4 + TanStack Router/Query on the frontend, embedded into the binary with rust-embed. Optional Tauri 2 desktop shell that bundles the server as a sidecar. Bun for JS tooling, just for tasks.

Two example resources are wired through every layer (migration, queries, API handlers, generated TypeScript types, UI page, tests): `items`, a minimal todo CRUD, and `posts`, which adds a status lifecycle (draft -> published -> archived), filtered queries with pagination, and an aggregate stats endpoint. Replace them with your real domain. The wiring pattern stays.

## Setup

Prerequisites: [Rust](https://rustup.rs), [Bun](https://bun.sh), and optionally [just](https://github.com/casey/just).

```bash
# 1. Rename the project (one time, the script deletes itself)
./scripts/setup.sh "My Project"

# 2. Install frontend deps and build everything
cd interface && bun install && bun run build && cd ..
cargo run
```

Open http://localhost:8080. The binary serves both the API and the built UI.

Copy `.env.example` to `.env` to override port, database path, or log level. Run `just hooks` once to enable the cargo fmt pre-commit hook.

## Development loop

Two terminals:

```bash
cargo run             # backend on :8080
just frontend-dev     # Vite on :5173, /api proxied to the backend
```

Hot reload on the frontend, restart the backend on Rust changes.

### The typegen loop

This is the core workflow. The backend is the single source of truth for API types:

1. Add or change a handler in `src/api/`, annotate it with `#[utoipa::path]`, register it in `src/api/mod.rs` (`paths` and `schemas`)
2. Run `just typegen`
3. The frontend client (`interface/src/api/client.ts`) is now fully typed for the new endpoint. Wrong paths, params, or body shapes fail `tsc`.

## Tasks

```bash
just dev            # run the backend
just frontend-dev   # run Vite with API proxy
just typegen        # regenerate TS types from the OpenAPI spec
just lint           # fmt check + clippy -D warnings + tsc, same as CI
just test           # backend tests (in-memory SQLite)
just build          # production build: frontend, then binary with UI embedded
just docker-build   # build the Docker image
```

## Project layout

```
src/
  main.rs          server entry (clap args, env via .env)
  lib.rs           AppState
  api/             HTTP layer: routes + OpenAPI annotations
  items.rs         example domain module: types + queries
  posts.rs         second example: status lifecycle, filtered queries, stats
  db.rs            pool init + migrations
  error.rs         AppError -> HTTP status mapping
  frontend.rs      embedded SPA serving with index.html fallback
  bin/openapi_spec.rs  prints the spec for typegen
migrations/        sqlx migrations, ordered by timestamp prefix
interface/         React app (Vite, Tailwind 4, TanStack)
  src/api/         generated schema.d.ts + typed fetch client
desktop/           Tauri 2 shell, server bundled as sidecar
tests/             black-box API tests via tower::oneshot
```

## Desktop app (optional)

The Tauri shell wraps the same UI and ships the server binary as a sidecar. Release builds spawn it on launch and kill it on exit. Needs the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
just desktop-dev      # run backend separately with `cargo run` first
just desktop-build    # bundles sidecar + frontend + installer
```

Before shipping: replace the placeholder icon if needed with `cd desktop && bunx tauri icon src-tauri/icons/icon.png`, and change the bundle identifier in `desktop/src-tauri/tauri.conf.json` from `com.example.*` to your reverse domain.

## Deploy

`Dockerfile` builds a multi-stage image: bun for the frontend, cargo for the binary, slim Debian runtime. SQLite lives on a volume at `/data`.

```bash
docker compose up --build
```

Works as-is on Coolify or any Docker host: point it at the repo, the Dockerfile does the rest. Pushing a `v*` tag publishes a multi-arch image to GHCR via `.github/workflows/release.yml`.

Note: CORS is permissive so the Tauri shell can reach the sidecar. Tighten `CorsLayer` in `src/api/mod.rs` before exposing the API publicly without the embedded UI.

## License

MIT
