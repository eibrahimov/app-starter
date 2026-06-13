# App Starter

Full-stack Rust starter. One binary serves the API and the UI, typed end to end via OpenAPI.

App Starter is for builders who want an explicit, shippable Rust + TypeScript foundation without framework magic: SQLite-first, backend-owned API types, a replaceable React UI, optional Docker/Tauri delivery, and examples that show the full vertical slice.

**Stack:** axum + SQLite (sqlx) on the backend. React 19 + Vite + Tailwind 4 + TanStack Router/Query on the frontend, embedded into the binary with rust-embed. Optional Tauri 2 desktop shell that bundles the server as a sidecar. Bun for JS tooling, just for tasks.

Two example resources are wired through every layer (migration, queries, API handlers, generated TypeScript types, UI page, tests): `items`, a minimal todo CRUD, and `posts`, which adds a status lifecycle (draft -> published -> archived), filtered queries with pagination, and an aggregate stats endpoint. Replace them with your real domain. The wiring pattern stays.

## Use this when

- You want one Rust server to own the API and serve the built UI.
- You want generated TypeScript types from the backend OpenAPI contract.
- You want SQLite and Docker to cover local development through small-team deployment.
- You prefer explicit files and examples over generators and hidden framework behavior.
- You may want a desktop shell later, but do not want desktop concerns to dominate the web/API core.

## Choose another path when

- You need built-in auth, tenancy, billing, or admin screens on day one.
- You need multiple database backends or a framework-neutral frontend choice.
- You want a Kubernetes/cloud-platform template more than a single-binary app foundation.
- You want a CRUD generator or no-code system instead of a small code template.

## Setup

Prerequisites: [Rust](https://rustup.rs), [Bun](https://bun.sh), and optionally [just](https://github.com/casey/just).

```bash
# 1. Rename the project (one-time fresh-template initialization; the script deletes itself)
./scripts/setup.sh "My Project"

# 2. Install frontend deps and build everything
cd interface && bun install && bun run build && cd ..
cargo run
```

Open http://localhost:8080. The binary serves both the API and the built UI.

Copy `.env.example` to `.env` to override port, database path, or log level. Run `just hooks` once to enable the cargo fmt pre-commit hook.

`scripts/setup.sh` is only for a fresh clone of the template. Do not run it during normal feature work or in an already-initialized generated project.

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

For the complete vertical-slice recipe, see [`docs/add-a-resource.md`](docs/add-a-resource.md).

## Tasks

```bash
just dev            # run the backend
just frontend-dev   # run Vite with API proxy
just typegen        # regenerate TS types from the OpenAPI spec
just check-typegen  # fail if committed TS types are stale, same as CI
just lint           # fmt check + clippy -D warnings + tsc, same as CI
just test           # backend tests (in-memory SQLite)
just build          # production build: frontend, then binary with UI embedded
just docker-build   # build the Docker image
```

For contribution gates, approval boundaries, and generated-project feedback loops, see [`CONTRIBUTING.md`](CONTRIBUTING.md). For the long-term template direction and v1 priorities, see [`docs/template-direction.md`](docs/template-direction.md).

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

Note: CORS is permissive so the Tauri shell can reach the sidecar. Tighten `CorsLayer` in `src/api/mod.rs` before exposing the API publicly without the embedded UI. See [`docs/production-readiness.md`](docs/production-readiness.md) before shipping a generated app to real users.

Generated projects can pull template fixes deliberately; see [`UPGRADING.md`](UPGRADING.md).

## License

MIT
