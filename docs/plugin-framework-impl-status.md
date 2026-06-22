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
- [ ] **0b — Foundations.** Add `utoipa-axum 0.2`; `src/plugin.rs` (`Plugin` trait + `PLUGIN_API_VERSION`); empty generated `src/plugins/mod.rs`; convert to Cargo workspace with `plugins/*` glob; add `run_all_migrators(pool)` and route BOTH `db::init()` and `tests/common.rs` through it; prove `default-run="app-starter"`, Docker `--bin app-starter`, and `build.rs` paths still work.
- [ ] **1 — Registry-driven router + typegen.** `router()`/`ApiDoc` fold in `plugins::all()`; build typegen spec from the server's own `router()` via `split_for_parts()`; repurpose the parity test; `just typegen` and commit `schema.d.ts`.
- [ ] **2 — `items` → first plugin (backend + frontend together).** Move `src/items.rs`, `src/api/items.rs`, its migration, and `interface/src/pages/Items.tsx` into `plugins/items/`; add generated `register()` line; add Vite `server.fs.allow` + tsconfig/biome scope; delete central registrations; `just typegen`.
- [ ] **3 — `posts` → second plugin** (backend + frontend together, same pattern).
- [ ] **4 — Authoring.** `add-plugin` skill + `just new-plugin` scaffolder (crate + migration + page + manifest AND appends to workspace + generated registry + Vite/tsconfig wiring); rewrite the `add-resource` recipe in `AGENTS.md`; add `docs/authoring-a-plugin.md`.

## Guard tests (spec §6 — implement and keep green)

- [ ] parity test reworked to a registry walk; `.nest`/`.merge` ban lifted.
- [ ] `typegen_spec_matches_server`
- [ ] `every_plugin_route_is_under_its_derived_prefix`
- [ ] `no_cross_plugin_schema_name_collisions`
- [ ] `plugin_tables_are_prefixed_and_unique`
- [ ] `expected_plugins_are_registered` — MUST run against a RELEASE (lto+strip) artifact in CI (smoke check on `/api/openapi.json`), not just debug.

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
