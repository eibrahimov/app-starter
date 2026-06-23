# Agent Guide

Full-stack Rust starter: one axum binary serves the JSON API and the embedded React
SPA. The backend is the single source of truth for API types. Resource endpoints live
under `/api/v1/*` (versioned); operational endpoints (`/api/health`, `/api/openapi.json`)
stay unversioned so probes and tooling have a stable path across versions.

The repository's founding intent is recorded in [VISION.md](VISION.md) — human-maintained;
reference it for intent, never edit it (see Hard rules).

Language-level conventions live in [RUST_STYLE_GUIDE.md](RUST_STYLE_GUIDE.md) and
[TS_STYLE_GUIDE.md](TS_STYLE_GUIDE.md); this guide covers the cross-cutting
workflow, the resource recipe, and approval boundaries.

The UI layer is Radix Themes (`@radix-ui/themes`); the single global config surface
is `interface/src/theme/theme.config.ts` (fed into `<Theme>`). See
[docs/radix-reference.md](docs/radix-reference.md) for the component catalog and
Theme props, and [docs/radix-workflow.md](docs/radix-workflow.md) for the
end-to-end lifecycle that takes a natural-language app request to a gated build.

## Validation matrix

Required before normal PR/commit handoff:

```sh
just lint      # cargo fmt --check + clippy -D warnings + frontend Biome + tsc
just test      # backend black-box tests against in-memory SQLite
just check-typegen  # fail if the committed types are stale (CI enforces this)
just verify    # everything CI runs: lint + test + typegen drift + frontend build/test + cargo-deny
```

Also run when relevant:

```sh
just doctor    # preflight: toolchain + frontend install/build drift; run on a fresh clone or when an env-related failure appears
just typegen   # when API/OpenAPI annotations changed; commit interface/src/api/schema.d.ts
just build     # release, embedded UI, Docker, frontend build, or packaging changes
just docker-build    # Dockerfile/compose/deployment changes
just desktop-build   # desktop/Tauri sidecar changes
just db-check        # SQLite integrity_check + applied migrations; after backup/restore or DB-file work (docs/recipes/backup-restore.md)
just db-selftest     # smoke-test scripts/db.sh (backup/restore/check round-trip on a throwaway DB); after editing the helper
just a11y      # opt-in accessibility page smoke (Playwright + axe); one-time `bunx playwright install chromium`. Non-blocking in CI.
```

Rust-only commands must set `SKIP_FRONTEND_BUILD=1` (build.rs otherwise shells out to
bun). JS tooling is bun/bunx only — never npm, pnpm, yarn, or npx. Run
`cd interface && bun install` once before lint/typegen on a fresh clone.

If a relevant validation command cannot be run, report why and state the remaining risk.

## Agent operating checklist

Before editing:

- Check `git status --short` and do not overwrite unrelated user changes.
- Identify the touched layer: migration, domain, API, typegen, frontend, desktop, CI, docs, or release.
- For multi-layer work, state the validation commands you expect to run.

While editing:

- Keep changes scoped to the request; do not refactor adjacent code unless required.
- Do not run `scripts/setup.sh` unless explicitly asked to initialize a fresh clone.
- Never hand-edit generated `interface/src/api/schema.d.ts`; run `just typegen`.
- Never edit or rename committed migrations.

Before handoff:

- Run the validation matrix commands that apply.
- Report commands run and results.
- State the reason and remaining risk for any skipped command.

## Approval boundaries

Agents may proceed without additional approval for:

- small bug fixes, tests, and docs clarifications;
- resource additions that follow the existing `items`/`posts` pattern.

Ask for human approval before changing:

- security posture, authentication, authorization, CORS, request limits, or public exposure defaults;
- dependency policy, major dependency upgrades, Rust edition/toolchain, or JS package manager;
- release workflow, Docker publishing, tags, package names, binary names, or registry targets;
- migration history, data-destructive behavior, or committed migration edits/renames;
- generated-file policy or hand-editing `interface/src/api/schema.d.ts`;
- architectural conventions not represented by both worked examples;
- license policy or third-party code/vendor naming.

Record the approval issue, PR, or comment in the change description.

## Adding a resource end to end

Resources are now self-contained **plugins** (docs/plugin-framework.md). You no
longer hand-edit the central router, OpenAPI document, migration list, or
`router.tsx` — a plugin declares what it contributes and the host builds the
router, spec, migrations, and nav by iterating the registered plugins. The two
worked examples are plugins: `todo` (minimal CRUD, `plugins/todo/`) and `blog`
(status lifecycle, filtered/paginated list, stats; `plugins/blog/`).

1. **Scaffold:** `just new-plugin <name>` (lowercase, e.g. `guestbook`). It
   generates the backend crate (`plugins/<name>/`: `Cargo.toml`, `plugin.toml`,
   `src/lib.rs`, a `migrations/<ts>_create_<name>_items.sql`), the frontend page
   (`interface/src/plugins/<name>/`), and makes the two central edits for you —
   the path dep in the root `Cargo.toml` and the `<name>::register()` line in the
   generated `src/plugins/mod.rs`.
2. **Customize** the generated migration, domain, handlers, and page for your
   schema, keeping the invariants the §6 guard tests enforce:
   - routes under `/api/v1/<name>` (from the handler `#[utoipa::path(path = ...)]`);
   - OpenAPI components prefixed `<name>_*` via `#[schema(as = <name>_Type)]`;
   - tables prefixed `<name>_*` (the plugin owns `plugins/<name>/migrations/`, its
     own `_sqlx_migrations_<name>` keyspace; migrations stay append-only — never
     edit a committed one).
   Handlers take `State(state): State<AppState>`, return `Result<_, AppError>`,
   validate input (`AppError::BadRequest`), and map a missing row to
   `AppError::NotFound`. Optional demo data goes through the `seed()` hook on the
   `Plugin` trait (see the todo/blog plugins) — core never seeds a plugin.
3. **Typegen:** `just typegen` and COMMIT the regenerated
   `interface/src/api/schema.d.ts` (never hand-edit it).
4. **Verify:** `just verify` — lint, backend tests (including the §6 namespacing
   guards in `tests/plugins.rs`: route-prefix, schema-prefix/no-collision,
   table-prefix, registered), typegen drift, frontend build/test, cargo-deny.

The frontend page lives under `interface/src/plugins/<name>/` (so it resolves the
shared Radix/hook deps and the typed `api` client the normal way) and is discovered
at build time by `interface/src/plugins/registry.ts`; the backend crate stays in
`plugins/<name>/`. Compose the page from Radix Themes + the `sections/` layer + the
typed hooks (`useApiQuery`/`useApiMutation`), exactly as `Todo.tsx`/`Blog.tsx` do —
see [docs/radix-reference.md](docs/radix-reference.md) and the `add-component` skill.

See [docs/authoring-a-plugin.md](docs/authoring-a-plugin.md) for the full
walkthrough and the `add-plugin` skill for the operational procedure. (The
pre-plugin central-registration recipe — and the now-legacy `add-resource` skill /
`docs/add-a-resource.md` — are superseded by this flow.)

## Feeding learnings back

Each generated app can improve the template. When work reveals reusable friction or a repeated pitfall:

- Update `AGENTS.md` if it changes how agents or contributors should work.
- Update `README.md` if it changes setup, development, deployment, or release behavior.
- Update `docs/add-a-resource.md`, `docs/components.md`, `docs/production-readiness.md`, `docs/desktop-features.md`, or `UPGRADING.md` when resource, frontend-component, hardening, desktop-shell, or upgrade guidance changes.
- Open an issue before adding a reusable pattern or changing template defaults.
- Keep generated-app/domain-specific details out of the template; port the pattern, not the feature.

Use `docs/contribution-prompts.md` for structured issue, change-request, backport, and PR prompts.

## Hard rules

- Migrations are append-only; new files must sort last.
- `interface/src/api/schema.d.ts` is generated only — regenerate, never edit.
- Do not remove the Tauri-aware baseUrl logic in `interface/src/api/client.ts` or the
  permissive `CorsLayer` in `src/api.rs` (the desktop sidecar needs both).
- The desktop shell is **desktop-only**: `desktop/src-tauri/src/main.rs` spawns the
  axum server as an `externalBin` sidecar and the webview talks HTTP to it. This model
  does NOT run on mobile — iOS forbids spawning child processes and Android fails
  bundled-binary exec with `os error 2` (tauri#9774, still open; maintainer "Not yet").
  Do NOT add iOS/Android bundle targets or wire mobile release/CI expecting the sidecar
  to start; a fork that runs `tauri ios build` gets a UI where every API call fails.
  Mobile requires the in-process-axum refactor first — and even then OS WebView
  cleartext/ATS policy blocks `http://localhost`, so it is not a turnkey path. See
  [docs/desktop-features.md](docs/desktop-features.md) for the full rationale and the
  ranked desktop-feature roadmap.
- Keep clippy clean: CI runs `-D warnings`; run `cargo fmt --all` before committing.
- Tests refer to the crate as `app_starter`; scripts use the `app-starter` binary
  name. `scripts/setup.sh` renames both — do not hardcode other variants.
- `scripts/setup.sh` (or `scripts/setup.ps1` on Windows) is only for fresh-template initialization; do not run it during normal feature, bugfix, or audit work.
- Code copied from external projects must be license-compatible (this template is
  MIT) and free of third-party product names; prefer clean-room reimplementation of
  patterns over copying files.
- API endpoints are versioned: new resource routes go under `/api/v1/`. Within v1,
  only ADD fields or endpoints — never remove or repurpose them (the generated TS
  client and any downstream consumer are pinned to the contract). A breaking change
  opens `/api/v2` alongside v1.
- Shared HTTP concerns (body-size limit, request timeout, request-id, CORS, graceful
  shutdown) live as layers in `src/api.rs::router` and `src/main.rs`, not per
  handler. `/api/health` is a readiness probe that pings the database — keep it cheap
  and side-effect free.
- `VISION.md` is human-maintained: AI agents MUST NOT edit, rewrite, reformat, or
  delete it — reference it for intent only. `.claude/settings.json` denies write
  access and `.github/CODEOWNERS` gates it to human review; changing the vision is a
  human-only action.
