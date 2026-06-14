# App Starter

Full-stack Rust starter. One binary serves the API and the UI, typed end to end via OpenAPI.

**Who it's for:** technical builders and AI coding agents who want a fast, type-safe foundation for shipping real software. It assumes Rust, Bun, and a terminal. Making this accessible to non-technical builders (guided scaffolding, AI-driven generation) is a deliberate next-phase goal on the [Roadmap](#roadmap), not a v1 claim.

**Vision:** the long-horizon intent behind this template is in [VISION.md](VISION.md) — human-maintained; agents reference it but never edit it.

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
# 1. Rename the project (one time, the script deletes itself)
./scripts/setup.sh "My Project"
# Windows (PowerShell): ./scripts/setup.ps1 "My Project"

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

Resource endpoints are versioned under `/api/v1/`, and the contract is additive within a version: add fields and endpoints freely, but a breaking change opens `/api/v2`. Health and the spec stay unversioned at `/api/health` and `/api/openapi.json`.

## Tasks

```bash
just dev            # run the backend
just frontend-dev   # run Vite with API proxy
just typegen        # regenerate TS types from the OpenAPI spec
just check-typegen  # fail if committed TS types are stale, same as CI
just lint           # fmt check + clippy -D warnings + tsc (fast gate)
just verify         # everything CI runs: lint + test + typegen + frontend build/test + cargo-deny
just test           # backend tests (in-memory SQLite)
just build          # production build: frontend, then binary with UI embedded
just docker-build   # build the Docker image
```

If you don't have `just`, run the commands directly — see the [`justfile`](justfile) for the exact recipe behind each task.

For contribution gates and approval boundaries, see [`CONTRIBUTING.md`](CONTRIBUTING.md); for template direction and v1 priorities, see [`docs/template-direction.md`](docs/template-direction.md).

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

The Tauri shell wraps the same UI and ships the server binary as a sidecar. The shell spawns it on launch and kills it on exit, in both dev and packaged builds, so a single command runs the whole app. Needs the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
just desktop-dev      # spawns the bundled backend automatically
just desktop-build    # bundles sidecar + frontend + installer
```

To iterate on the backend with hot reload instead, run `cargo run` at the repo
root first; the shell's sidecar then fails to bind the port and exits while your
`cargo run` serves the window.

Before shipping: replace the placeholder icon if needed with `cd desktop && bunx tauri icon src-tauri/icons/icon.png`, and change the bundle identifier in `desktop/src-tauri/tauri.conf.json` from `com.example.*` to your reverse domain.

## Database

SQLite via sqlx — a single file on disk (or `:memory:` in tests). It is a **single-writer** database: ideal for single-instance apps, internal tools, and desktop, but it does not support multiple server instances writing concurrently. For multi-instance or high-write deployments, swap sqlx to Postgres — that touches the pool type in `src/db.rs`, every `query`/`query_as`, and the migrations, so treat it as a fork, not a config flag. Migrations are forward-only and append-only: to fix a bad migration, add a new one; never edit a committed file (sqlx checksums them).

## Troubleshooting

- **`bun: command not found`** during build: install [Bun](https://bun.sh), then `cd interface && bun install`.
- **Port 8080 already in use:** run `PORT=8081 cargo run`, or stop the other process.
- **`frontend not built` page:** run `cd interface && bun install && bun run build`, then `cargo run`.
- **`tsc` errors after changing an endpoint:** run `just typegen` and commit `interface/src/api/schema.d.ts` — it is generated, never hand-edited.
- **`database is locked`:** another process holds the SQLite file; stop it, or point `DATABASE_URL` at a different path.

## Deploy

`Dockerfile` builds a multi-stage image: bun for the frontend, cargo for the binary, slim Debian runtime. SQLite lives on a volume at `/data`.

```bash
docker compose up --build
```

Works as-is on Coolify or any Docker host: point it at the repo, the Dockerfile does the rest. Pushing a `v*` tag publishes a multi-arch image to GHCR and attaches prebuilt Linux/macOS/Windows binaries to the GitHub Release via `.github/workflows/release.yml`.

Note: CORS is permissive so the Tauri shell can reach the sidecar. Tighten `CorsLayer` in `src/api/mod.rs` before exposing the API publicly without the embedded UI.

## Roadmap

v1 ships and supports today: **Web** (embedded SPA), **Docker** (multi-arch image to GHCR on `v*` tags), **Desktop** (macOS/Windows/Linux installers built locally via `just desktop-build`), **prebuilt binaries** (Linux/macOS/Windows) attached to the GitHub Release on each `v*` tag, and a **Vitest** frontend test harness with non-blocking coverage reporting.

Planned, explicitly post-v1 (tracked as issues — contributions welcome):

- **Signed desktop installers** in CI (macOS notarization, Windows code-signing) — requires developer certificates.
- **Mobile (iOS/Android)** via Tauri 2 — icons are present, but build/signing/store wiring is not. Do not assume `tauri build ios`/`android` works yet.
- **Accessibility for non-technical builders** — guided scaffolding and AI-assisted generation.

## License

MIT
