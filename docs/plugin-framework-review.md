# Plugin Framework — Consolidated Re-Review

> Output of a 6-agent adversarial review panel that verified
> [`plugin-framework.md`](plugin-framework.md) and
> [`plugin-framework-research.md`](plugin-framework-research.md) against the
> **actual repo** (pinned deps, `Cargo.lock`, guard tests, build config) and
> **primary sources** — not at face value.
>
> **Date:** 2026-06-22. **Panel:** backend/Rust · DB/migrations · frontend ·
> citation audit · security/governance · consistency/process.
>
> **Verdict:** the *architecture* is sound (compile-time registration,
> `utoipa-axum`, build-time UI discovery, crate plugins are each validated), but
> **5 BLOCKERS gate sign-off** — three of them contradict the doc's headline
> promises ("no central edits", "typed end to end", "every phase stays green"),
> and two are flat factual errors (`dangerous_set_table_name` doesn't exist in
> the pinned sqlx; the auto-registration model is unsound under Rust's linker).
> None require abandoning the approach; all are fixable. Citation integrity of
> the research is high; its defects are peripheral.

## 1. Severity summary

| # | Finding | Sev | Panel | Contradicts a stated promise? |
|---|---------|-----|-------|------|
| B1 | "A plugin is enabled just by being a dependency" is false — an unreferenced rlib is dropped, so `inventory::submit!` never runs; author must explicitly link each plugin | **BLOCKER** | backend + consistency | Yes — "no central edits"/"implicit enable" |
| B2 | `Migrator::dangerous_set_table_name` does **not exist in sqlx 0.8.6** (the pinned version) — only 0.9.0+ | **BLOCKER** | DB | Yes — "settable today" is false |
| B3 | Test harness `tests/common.rs` runs `migrate!("./migrations")` directly, never `db::init()` → moving items/posts migrations into plugins breaks `just verify` | **BLOCKER** | consistency | Yes — "each phase stays green" |
| B4 | A runtime-built route array forfeits TanStack `<Link>` compile-time type safety | **BLOCKER** | frontend | Yes — "typed end to end" |
| B5 | Vite dev server 403s on `../../../plugins/...` without a `server.fs.allow` edit to `vite.config.ts` | **BLOCKER** | frontend | Yes — "no central edit" |
| M1 | `inventory` silent-loss is a *class* (module placement, rustc ver, `lld`, MSVC, LTO) and the **shipping** profile (`lto=thin`+`strip`) is the risk-prone one; the presence guard runs only in debug | MAJOR | backend + security + consistency | — |
| M2 | Typegen binary (`src/bin/openapi_spec.rs`) is a separate link unit → generated `schema.d.ts` can silently diverge from what the server serves | MAJOR | backend | — |
| M3 | Cross-plugin migration **ordering / FK-to-core / partial-failure** undesigned; per-plugin tracking-table name collision unguarded | MAJOR | backend + DB | — |
| M4 | SQLite `SQLITE_BUSY`/locking at multi-migrator startup unaddressed (no WAL, no `busy_timeout` in `src/db.rs`) | MAJOR | DB | — |
| M5 | ATTACH-per-file (SQLite's native isolation primitive) never considered as the migration alternative | MAJOR | DB | — |
| M6 | Supply-chain evidence mismatched: cited crates.io incidents (`faster_log`, `rustdecimal`) executed at **runtime**, not build time — they don't support the `build.rs` thesis | MAJOR | security + citation | — |
| M7 | Trust model stops one sentence short: never states an in-process plugin can read the DB file, env/secrets, and open network egress **outside** the HTTP layers | MAJOR | security | — |
| M8 | Governance §2.5 imports runtime-marketplace concepts (per-version scanning, featured tiers, install counts) that don't map to a compile-time crate model with no install step | MAJOR | security | — |
| M9 | Lifting the `.nest`/`.merge` ban drops the parity test's **Direction-3 shadowing** protection without justifying it | MAJOR | consistency | — |
| M10 | Plugin TSX lives outside `interface/` → `tsc --noEmit` and Biome **skip it**; Phase 4 *reduces* gate coverage | MAJOR | frontend | Yes — "same gates" |
| — | ~12 MINOR/NIT items (stale §4 table row, `register!` vs `register_plugin!`, `api_prefix` triple-source, eager-glob kills code-splitting, undefined `PluginRoute`, missing approval boundaries, imprecise citations) | minor | all | — |

## 2. The five blockers, in detail

### B1 — Auto-registration is unsound as stated; the "no central edits" promise is overstated
**(backend B1 + consistency M1 — same root cause.)** `inventory::submit!` plants a
life-before-main constructor kept with `#[used]`, but `#[used]` only protects a
symbol *inside an object file the linker actually pulls in*. Rust links upstream
crates as **rlibs** (`ar` archives); a dependency crate from which the final
binary references **no symbol** can be dropped wholesale — constructors and all.
So "a plugin is on if its crate is a dependency" (§3, §10-Q1) is false, and the
proposed runtime presence test (§6) only *detects* the empty registry, it
doesn't *make the plugin link*. Confirmed: rust-ctor#27 (a `#[ctor]` in a
dependency "is not called… does not seem to be included"; fix = force-reference a
symbol), inventory issues #11/#32/#34/#52/#85.

Corollary the doc hides: for `inventory` to work, the host **must** name each
plugin crate (workspace `members` + a binary `[dependencies]` entry + an explicit
symbol reference). Those are central-file edits. The core promise must narrow to:
*"a plugin author never edits `src/api.rs`, the OpenAPI doc, `router.tsx`, or the
migration list"* — the **build-graph edit remains**.

**Fix:** explicit, symbol-referencing registration — the scaffolder/core generates
a `plugins/mod.rs` with `pub use <plugin>;` + a `force_link()` call per plugin (or
calls each plugin's `register()` directly and drops `inventory` entirely).
Downgrade §10-Q1's "implicit" option to a documented anti-pattern.

### B2 — `dangerous_set_table_name` is not in sqlx 0.8.6 (the pinned version)
**(DB panel, verified at release tags.)** `Cargo.lock` pins `sqlx 0.8.6`. The
`impl Migrator` in `v0.8.6/sqlx-core/src/migrate/migrator.rs` has only
`new, set_ignore_missing, set_locking, iter, version_exists, run, undo` — **no**
`dangerous_set_table_name`. That method first ships in **0.9.0** (2026-05-06).
So §5's "settable today" and research §2.3's "achievable today" are **false
against the repo**. The per-plugin-tracking-table default cannot be implemented
without a **major, breaking sqlx upgrade** (0.8→0.9 changed `set_*` signatures,
added `sqlx.toml`), which is itself an `AGENTS.md`-gated dependency change the doc
never names.

**Fix:** pick one and own it — (a) gate the design on `sqlx >= 0.9` and route the
upgrade through approval with a breaking-change audit; (b) stay on 0.8 with a
0.8-compatible path: a custom per-plugin versioned-SQL applier writing to
`<plugin>_schema_migrations`, **or** ATTACH-per-file (M5, each file gets its own
default `_sqlx_migrations`, no 0.9 API needed). Note the corrected SQLite caveat:
ATTACH can't cross-DB-FK to core tables and loses cross-file atomicity under WAL.

### B3 — The test harness never runs `db::init()`, so the "green phases" claim fails
**(consistency panel.)** `tests/common.rs::memory_pool()` calls
`sqlx::migrate!("./migrations")` **directly**; it does not go through `db::init()`.
The moment Phase 2/3 moves the items/posts `.sql` files into `plugins/*/migrations/`,
the test DB no longer creates those tables and `items_crud_roundtrip`,
`posts_lifecycle_roundtrip`, the parity test, and the CHECK-constraint tests all
fail — directly contradicting §7's "each phase is independently shippable and
keeps `just verify` green."

**Fix:** add an explicit phase step (and a §4 row) to route the test harness through
a shared `run_all_migrators(pool)` that both `db::init()` and `tests/common.rs` call.

### B4 — Runtime-built route array forfeits `<Link>` type safety
**(frontend panel, verified against TanStack docs.)** TanStack derives type-safe
navigation from a **statically-known** route tree. `pluginRoutes: PluginRoute[]`
erases the per-path literals, so `<Link to="/guestbook">` degrades to `string` and
dead links aren't caught at compile time. TanStack docs are explicit: routes built
in a way TS "will not be able to infer… meaning you'd have to manually type the
`to` prop… and wouldn't catch errors until runtime." The current `router.tsx` is
type-safe *because* it lists static literals. The "stays typed end to end" claim is
true for the **API client** (openapi-fetch) but false for **navigation**.

**Fix:** scope the typed guarantee to the API client; add a runtime guard that every
`pluginRoutes[].path` resolves; or emit a generated static route-tree module — which
reintroduces a codegen step (contradicting "no codegen", so state the trade-off).

### B5 — Vite dev server 403s on the glob target without a central config edit
**(frontend panel, verified.)** The Vite root is `interface/`; `plugins/` is a
sibling, outside the served root. `server.fs.strict` defaults true and Vite 403s on
files outside the workspace root. Auto workspace-root detection needs a
`workspaces`/`pnpm-workspace.yaml`/`lerna.json` marker — **none exist here** — so the
served root stays `interface/`. The glob therefore needs `server.fs.allow: ['..']`
(or `searchForWorkspaceRoot`) in `vite.config.ts`. Production `vite build` is
unaffected (Rollup bundles anywhere), but this is a required central edit the doc
omits.

**Fix:** add the `vite.config.ts` `server.fs.allow` change to §3/§4; pin the glob as
relative (an absolute `/plugins` resolves to the Vite root, not repo root, and would
not match).

## 3. Majors worth fixing before implementation

- **M1 (release-profile guard):** make `expected_plugins_are_registered` run against
  a **release `lto=thin`+`strip`** artifact in CI on every shipped target (Linux/
  macOS/Windows + Tauri sidecar); a debug-only assertion can't catch a release-only
  DCE drop. Document `lld`/MSVC/LTO as tested-or-unsupported.
- **M2 (typegen divergence):** generate the OpenAPI spec from the **server's own**
  `router()` via `split_for_parts()`, not a separate `ApiDoc` in the bin; add a guard
  that the typegen path/schema set equals the server's.
- **M3 (migration ordering):** core migrates first; a plugin may FK only to
  already-migrated core tables; derive the tracking-table name from the validated
  plugin id; add a guard that plugin **table names** and tracking-table names are
  unique (the §6 guards cover routes/OpenAPI components, **not** DB tables); document
  partial-failure behavior.
- **M4 (locking):** set `busy_timeout` (and likely WAL) before migrating; declare
  concurrent-instance startup against one file unsupported.
- **M6 (evidence):** decouple build-time from runtime risk. `build.rs`/proc-macros run
  arbitrary code at build time *by design* (cite generically); cite `faster_log`/
  `rustdecimal` only for the **runtime** in-process risk. Fix the `rustdecimal` date
  (2022, not 2025).
- **M7 (honest trust):** add one sentence — *a plugin can read the SQLite file, env/
  secrets, and open its own listeners/egress with full process authority; the CORS/
  body-limit/timeout layers govern only HTTP through the shared router and provide no
  containment. "Trusted plugins only" is the sole real control.*
- **M8 (governance fit):** reframe §2.5 for a compile-time model — the only actionable
  controls are PR review of the added dependency + `cargo-deny` (already in
  `just verify`) + optional `cargo-vet` on the workspace's own graph. Label per-version
  scanning / featured tiers / install counts as **not applicable** unless the WASM
  escape hatch is taken.
- **M9 (parity Direction-3):** state that `utoipa-axum`'s single-declaration model makes
  the route↔spec shadowing class impossible-by-construction (so dropping Direction-3 is
  safe) — or keep a source-level guard. Don't silently drop a named protection.
- **M10 (gate scope):** extend `interface/tsconfig.json` `include` and Biome config to
  cover `plugins/*/frontend`, or give plugins their own configs that `just lint`
  iterates. Otherwise Phase 4 reduces coverage.

## 4. What the review *confirmed* (de-risked)

- **`utoipa-axum` is real and compatible:** 0.2.0 ↔ axum 0.8.4, utoipa 5.0.0, MSRV 1.88
  — all match the repo. `OpenApiRouter`/`routes!`/`nest`/`merge`/`split_for_parts`
  behave exactly as the doc describes. Lifting the `.nest`/`.merge` ban is supported.
  (Note: §9 items "new deps" and "lift the ban" are **one coupled decision** —
  adopting `utoipa-axum` *requires* `.nest`/`.merge`.)
- **The `Plugin` trait is object-safe**; `AppState: Clone+Send+Sync+'static` makes the
  per-plugin `OpenApiRouter<AppState>` merge type-check and preserves the shared layers
  + SPA fallback (assembly order must be pinned: merge all routers → `split_for_parts`
  → attach fallback + layers last; guard that no plugin sets a fallback).
- **The migration *diagnosis* is correct:** the shared-`_sqlx_migrations` `VersionMissing`
  failure (sqlx#1698/#3573) and `ignore_missing` masking are real and accurately cited.
  The *prescription* is what needs the B2/M3/M4/M5 fixes.
- **Frontend build-time discovery + no-sandbox trust framing** is honest and the
  mechanism works for `vite build`; only the dev-server `fs.allow` and the type-safety
  scoping need fixing.
- **Citation integrity is high** (see §5).

## 5. Citation audit summary

Of ~28 load-bearing claims independently web-checked: the large majority **VERIFIED**
against the named source, including the specific/surprising ones (grack.com
"life-before-main" 2026-06-11; TanStack 84-versions/2026; arXiv 5.61%/613M; Chrome
39.8%; sqlx `VersionMissing`; Kubernetes one-storage-version; VS Code "cannot publish
proposed API"; Grafana node-semver; go-plugin protocol version; Obsidian "cannot
reliably restrict"). **No fabricated or dead URLs.** Defects are peripheral and move
**no design decision**:

- `rustdecimal` mislabeled a "2025" takedown — it's **2022** (RUSTSEC-2022-0042). *Fix.*
- "~90 sources" is inflated — **~57 unique URLs** (the 2× is inline-list + reference
  defs). *Soften to ~55+.*
- "silent last-wins" pinned to Redocly, which actually errors loudly; the risk is real
  for **naive/`.merge()`** paths (the design's own path), so the namespacing decision
  stands — fix the attribution.
- sqlx#3565 "schema per app" is a looser gloss than the source (it emphasizes changing
  the *migrations table name*) — which, if anything, supports the per-plugin-table
  decision *more* strongly.
- TanStack 2026 was a CI/Actions/OIDC compromise ("No npm tokens were stolen") — "via CI
  token" is right, "maintainer-takeover" is the wrong bucket. *Relabel.*

## 6. Decisions required before implementation

These are genuinely the maintainer's call and each unblocks a blocker/major:

1. **Registration (B1):** explicit generated `force_link`/`register()` per plugin
   (recommended) — and accept that "no central edits" narrows to "no edits to
   `api.rs`/OpenAPI/`router.tsx`/migrations." Confirm?
2. **sqlx (B2/M5):** upgrade to `sqlx 0.9` (gets `dangerous_set_table_name`, breaking,
   approval-gated) **vs.** stay on 0.8 with ATTACH-per-file **vs.** a custom per-plugin
   migration log. Which?
3. **Frontend typing (B4):** accept "API client typed, navigation runtime-checked"
   **vs.** add route-tree codegen (drops the "no codegen" claim). Which?
4. **First PR scope:** still machinery-only (Phases 0–1), now that the workspace
   conversion + explicit-registration + typegen-from-server are bigger than "green by
   default" implied?

---

*This review supersedes the "Open questions" framing in `plugin-framework.md` §10 for
the items it resolves. The design remains viable; it needs the five blockers closed and
the four decisions above made before Phase 0 begins.*
