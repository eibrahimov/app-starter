# Plugin Framework — Implementation Status (Ledger)

> Durable progress state for the plugin-framework migration loop.
> Spec: [`plugin-framework.md`](plugin-framework.md) (v2, authoritative).
> Branch: `claude/plugin-framework-impl` → PR base `claude/modular-plugin-framework-8ull11`.
>
> **Each iteration:** read this ledger + the spec, pick the FIRST unchecked unit,
> implement only it, run the per-unit cycle, check it off, report.

## Environment (verified iteration 1, 2026-06-23)

- Toolchain present: `cargo` (Rust 1.96.0), `just`, `bun`, `cargo-deny`.
- `sqlx 0.9.0` is published on crates.io (latest) — Phase 0a is feasible.
- Current pinned dep: `sqlx = "0.8"` (Cargo.toml:15).

## Phases (spec §7 — do in order)

- [x] **0a — sqlx 0.8→0.9 upgrade.** Bump dep; fix API breaks; `cargo deny` clean; `just verify` green. _(Done iter 1.)_
- [x] **0b — Foundations.** _(Done iter 2.)_ Add `utoipa-axum 0.2`; `src/plugin.rs` (`Plugin` trait + `PLUGIN_API_VERSION`); empty generated `src/plugins/mod.rs`; convert to Cargo workspace with `plugins/*` glob; add `run_all_migrators(pool)` and route BOTH `db::init()` and `tests/common.rs` through it; prove `default-run="app-starter"`, Docker `--bin app-starter`, and `build.rs` paths still work.
- [x] **0c — Plugin-API crate (cycle fix, iter 4).** Extracted `app-starter-plugin-api` (Plugin trait + AppState + PLUGIN_API_VERSION) so host and plugins depend on it without a cycle; `app-starter` re-exports it and keeps the registry.
- [x] **1 — Registry-driven router + typegen.** _(Done iter 3.)_ `router()`/`ApiDoc` fold in `plugins::all()`; build typegen spec from the server's own `router()` via `split_for_parts()`; repurpose the parity test; `just typegen` and commit `schema.d.ts`.
- [x] **2 — `items` → first plugin → `todo` (backend + frontend).** _(Done iter 5.)_ Move `src/items.rs`, `src/api/items.rs`, its migration, and `interface/src/pages/Items.tsx` into `plugins/items/`; add generated `register()` line; add Vite `server.fs.allow` + tsconfig/biome scope; delete central registrations; `just typegen`.
- [ ] **3 — `posts` → second plugin** (backend + frontend together, same pattern).
- [ ] **4 — Authoring.** `add-plugin` skill + `just new-plugin` scaffolder (crate + migration + page + manifest AND appends to workspace + generated registry + Vite/tsconfig wiring); rewrite the `add-resource` recipe in `AGENTS.md`; add `docs/authoring-a-plugin.md`.

## Guard tests (spec §6 — implement and keep green)

- [x] parity test reworked to a registry walk; `.nest`/`.merge` ban lifted. _(iter 3)_
- [x] `typegen_spec_matches_server` _(iter 3)_
- [ ] `every_plugin_route_is_under_its_derived_prefix`
- [ ] `no_cross_plugin_schema_name_collisions`
- [ ] `plugin_tables_are_prefixed_and_unique`
- [ ] `expected_plugins_are_registered` — MUST run against a RELEASE (lto+strip) artifact in CI (smoke check on `/api/openapi.json`), not just debug.

## Decisions (locked in-loop)

- **Phase 2/3 example naming (iter 4):** the worked-example plugins are renamed to
  avoid the name==table collision — `items` → **`todo`** (table `todo_items`,
  routes `/api/v1/todo`). `<plugin>_<entity>` models the namespace convention
  cleanly and satisfies the prefix invariant without an `items_items` oddity.
  Consequence: the committed `items` migration is replaced by a fresh plugin
  migration; pre-existing dev DBs need recreation (fresh-DB template — acceptable).
- **Plugin seed (iter 4):** add an optional `seed()` hook to the `Plugin` trait so
  each plugin owns its seed data (core's seed runner iterates `plugins::all()`),
  honoring "core never depends on a plugin."
- **Frontend location (iter 5):** the spec's "plugin frontend in
  `plugins/<name>/frontend/` importing interface deps" is **not buildable** —
  `node_modules` lives only in `interface/`, and Vite/vitest/tsc resolve a file's
  bare imports by walking up from its dir, never reaching `interface/node_modules`
  (a second spec flaw, after the Cargo cycle). Per decision, plugin **frontends
  live under `interface/src/plugins/<name>/`** (resolve shared deps + the typed
  client normally; no JS workspace); the backend **crate** stays in
  `plugins/<name>/`. `AppError` also moved to `plugin-api` (plugins return it).

## BLOCKER (iter 4) — circular package dependency — RESOLVED (Option A, phase 0c)

The v2 design is **not buildable as written**. It places the generated registry
in `app-starter`'s `src/plugins/mod.rs` (so `app-starter` depends on each plugin
crate) while every plugin depends on `app-starter` for `Plugin`/`AppState`. Cargo
forbids that cycle — **confirmed** via probe: `error: cyclic package dependency:
package 'app-starter' depends on itself`. Phase 0b only compiled because the
registry was empty. Resolution requires an architecture change (awaiting decision):
- **Option A (recommended):** extract a small `app-starter-plugin-api` crate
  (`Plugin` trait + `AppState` + `PLUGIN_API_VERSION`); plugins depend on it;
  `app-starter` keeps the bins + `src/plugins/mod.rs` registry and depends on
  plugin-api + each plugin; re-export the trait/state so `app_starter::*` keeps
  working. No cycle; bins/Docker/default-run/build.rs unchanged.
- **Option B:** make `app-starter` lib-only (router/seed/migrators take the plugin
  list as a param) and move the bins + registry into a new composition crate that
  depends on the lib + plugins. Matches `impl app_starter::Plugin` exactly but
  moves the binaries (re-prove §9 invariants).

## Stop condition

All phases + all guards checked, `just verify` green, release-profile registration
smoke check passes, final per-unit cycle clean, draft PR updated with an
"IMPLEMENTATION COMPLETE" summary.

## Iteration log

- **Iter 1 (2026-06-23):** Bootstrap + **Phase 0a complete**. Created branch
  `claude/plugin-framework-impl` and this ledger; verified toolchain + sqlx 0.9
  availability. Bumped `sqlx 0.8 → 0.9` (Cargo.toml). The only 0.8→0.9 break in
  current code was the new **`SqlSafeStr`** injection guard rejecting `&format!`
  query strings — fixed by wrapping the 3 audited dynamic reads in `src/posts.rs`
  with `AssertSqlSafe` (SQL composed only from the `SELECT_COLUMNS` const + literal
  SQL; all data still bound via `?n`). No `set_*` calls exist yet (those arrive in
  0b). Bumped declared `rust-version` 1.88 → 1.94 (sqlx 0.9 proc-macros require it).
  Gates green: `just verify` (backend + 155 FE tests + typegen drift clean) +
  `cargo deny` (advisories/bans/licenses/sources ok). Next unit: **Phase 0b**.
- **Iter 2 (2026-06-23):** **Phase 0b complete.** Added `utoipa-axum 0.2`;
  `src/plugin.rs` (`Plugin` trait — object-safe; `name`/`host_api`/`api`/`migrator`
  — + `PLUGIN_API_VERSION = "1.0.0"`), re-exported at crate root; empty generated
  `src/plugins/mod.rs` (`all() -> vec![]`); converted to a Cargo workspace
  (`members=["plugins/*"]`, resolver 2) with a `.gitkeep`-tracked `plugins/` dir
  (dotfile is not matched by the glob). Added `db::run_all_migrators(pool)` (sets
  `busy_timeout`+WAL, runs core migrator, then each plugin into
  `_sqlx_migrations_<name>` via `dangerous_set_table_name`, naming the failing
  plugin per §5.4); routed BOTH `db::init()` and `tests/common.rs` through it [B3].
  Router/OpenAPI still wired the old way (Phase 1). Proved invariants:
  `cargo build --bin app-starter` (SKIP_FRONTEND_BUILD) ok, `default_run=app-starter`,
  `default_members`=root, build.rs frontend build ran in `just verify`. Gates:
  `just verify` + `cargo deny` green; live smoke (WAL active) passed. Next: **Phase 1**.
- **Iter 3 (2026-06-23):** **Phase 1 complete** + 2 guards. Rewrote `src/api.rs`:
  `ApiDoc` is now info-only; `api_router()` builds an `OpenApiRouter` (core
  resources via `routes!`, plugins via `.merge(plugin.api())`) and
  `split_for_parts()` yields the live router + spec from one declaration. Added
  `pub api::api_spec()`; the `openapi_spec` bin + `/api/openapi.json` handler both
  use it [M2]. Reworked the parity test to a registry walk (probe every spec op →
  served); **lifted the `.nest`/`.merge` ban** and dropped the source parser +
  Direction 2/3 (impossible by construction, [M9]); added
  `typegen_spec_matches_server`. `just typegen` produced **zero `schema.d.ts`
  drift** — the router-derived spec is byte-identical to the old hand-built one.
  Gates: `just verify` + `cargo deny` green; both guard tests pass explicitly.
  Next: **Phase 2** (items → first plugin).
- **Iter 4 (2026-06-23):** Surfaced + resolved two blockers, then **landed phase
  0c (plugin-api extraction)**. User decisions: rename example `items` → `todo`
  (table `todo_items`); add a `seed()` hook to the `Plugin` trait (both apply in
  Phase 2). Then hit a fatal blocker — the spec's registry-in-core design forms a
  Cargo cycle (confirmed by a probe: `app-starter` ⇄ `todo`). User chose **Option
  A**: extracted `app-starter-plugin-api` (trait + AppState + PLUGIN_API_VERSION);
  `app-starter` re-exports it and keeps `src/plugins/mod.rs`. cargo-deny needed
  `allow-wildcard-paths = true` + `publish = false` on the internal crates so the
  intra-workspace path deps pass the wildcard ban. Registry still empty → behavior
  unchanged. Gates: `just verify` + `cargo deny` green. **Phase 2 (items → todo
  plugin) is now unblocked** and is the next unit.
- **Iter 5 (2026-06-23):** **Phase 2 complete** (items → `todo` plugin, backend +
  frontend). Hit + resolved a 2nd spec flaw (plugin frontend can't resolve
  interface deps from `plugins/<name>/frontend/`); per decision, frontends live
  under `interface/src/plugins/<name>/`. Built `plugins/todo` crate (domain +
  handlers `/api/v1/todo` + migration `todo_items` + `seed()`); moved `AppError`
  + added `seed()` hook to `plugin-api`; registered `todo::register()`; removed
  central items; built FE plugin discovery (`contract`/`registry`/`registry.test`
  + lazy router) and the `todo` page + tests under `interface/src/plugins/todo/`;
  repointed hook tests. `just typegen` → `/api/v1/todo`. `just verify` fully green
  (lint, 16 api tests incl. `todo_crud_roundtrip`, typegen clean, FE build with
  Todo code-split + 157 vitest, cargo-deny). Next: **Phase 3** (posts → plugin).
