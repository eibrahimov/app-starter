# Agent Guide

Full-stack Rust starter: one axum binary serves the JSON API under `/api/*` and the
embedded React SPA. The backend is the single source of truth for API types.

## Gate commands (run before any commit)

```
just lint      # cargo fmt --check + clippy -D warnings + frontend tsc
just test      # backend black-box tests against in-memory SQLite
just typegen   # regenerate interface/src/api/schema.d.ts from the OpenAPI spec
just check-typegen  # fail if the committed types are stale (CI enforces this)
just build     # production build: frontend, then release binary with UI embedded
```

Rust-only commands must set `SKIP_FRONTEND_BUILD=1` (build.rs otherwise shells out to
bun). JS tooling is bun/bunx only — never npm, pnpm, yarn, or npx. Run
`cd interface && bun install` once before lint/typegen on a fresh clone.

## Adding a resource end to end

Two worked examples are wired through every layer: `items` (minimal CRUD) and
`posts` (status lifecycle with transition validation, filtered list queries with
pagination, get-by-id, and an aggregate stats endpoint). Copy their shape exactly:

1. Migration: `migrations/<YYYYMMDDHHMMSS>_<description>.sql`. The timestamp must sort
   after every existing migration. NEVER edit or rename a committed migration — sqlx
   checksums them and the app will refuse to start against an existing database.
   Conventions: TEXT uuid primary key generated app-side, INTEGER booleans,
   TEXT timestamps, index on sort columns.
2. Domain module: `src/<resource>.rs` — row struct deriving
   `Debug, Serialize, sqlx::FromRow, utoipa::ToSchema`; payload structs deriving
   `Debug, Deserialize, ToSchema`; plain `async fn(pool: &SqlitePool, ...) ->
   Result<_, sqlx::Error>` queries. By-id mutations return `Result<bool, _>` from
   `rows_affected()`; no repository structs or traits.
3. Declare `pub mod <resource>;` in `src/lib.rs`. Extend `AppState` only for genuinely
   shared values; the pool usually suffices.
4. Handlers: `src/api/<resource>.rs` — thin functions taking
   `State(state): State<AppState>`, returning `Result<_, AppError>`. Every handler
   carries `#[utoipa::path(...)]` with the FULL literal path including the `/api`
   prefix, a per-resource `tag`, `params` for path/query inputs, `request_body` for
   POST bodies, and every response status with `body =` schema. Validate input in the
   handler (`AppError::BadRequest`); map a `false`/`None` store result to
   `AppError::NotFound`; 201 + Json for create, `StatusCode::NO_CONTENT` for
   delete-like actions.
5. Register in `src/api/mod.rs` in THREE places: (a) `pub mod <resource>;`, (b) the
   handler in the `#[openapi(paths(...))]` list AND every new ToSchema type in
   `components(schemas(...))`, (c) a `.route("/api/...", ...)` entry placed before
   `.fallback(crate::frontend::spa)`, using axum 0.8 brace syntax (`{id}`).
   An endpoint missing from (b) does NOT error — it silently vanishes from the
   OpenAPI spec and the generated TypeScript types. This is the most common mistake.
6. Tests: extend `tests/api.rs` reusing `test_app()` and `body_json()`; import the
   crate as `app_starter::...`. Cover the happy-path roundtrip plus one 400 and one
   404 case. Run `just test`.
7. Run `just typegen` and COMMIT the regenerated `interface/src/api/schema.d.ts`.
   Never hand-edit that file. Skipping typegen makes `just lint` (tsc) fail.
8. Frontend page: `interface/src/pages/<Name>.tsx` modeled on `Items.tsx` — data
   access only through the typed `api` client from `../api/client` (never raw
   `fetch`), `useQuery` with array keys, mutations that `invalidateQueries` on
   success, explicit isLoading/isError branches, inline Tailwind classes in the zinc
   palette, spaces not tabs, no `@/` path aliases.
9. Route: in `interface/src/router.tsx` add a `createRoute({...})`, append it to
   `rootRoute.addChildren([...])`, and add a nav `<Link>` in `Layout`.

## Hard rules

- Migrations are append-only; new files must sort last.
- `interface/src/api/schema.d.ts` is generated only — regenerate, never edit.
- Do not remove the Tauri-aware baseUrl logic in `interface/src/api/client.ts` or the
  permissive `CorsLayer` in `src/api/mod.rs` (the desktop sidecar needs both).
- Keep clippy clean: CI runs `-D warnings`; run `cargo fmt --all` before committing.
- Tests refer to the crate as `app_starter`; scripts use the `app-starter` binary
  name. `scripts/setup.sh` renames both — do not hardcode other variants.
- Code copied from external projects must be license-compatible (this template is
  MIT) and free of third-party product names; prefer clean-room reimplementation of
  patterns over copying files.
