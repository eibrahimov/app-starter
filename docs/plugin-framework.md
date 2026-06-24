# Plugin Framework — Design Doc (v2)

> **Status:** Proposal (design only — no code changes yet). Awaiting sign-off
> before implementation. Deliverable for the `claude/modular-plugin-framework`
> branch.
>
> **v2 — revised after re-review.** A 6-agent adversarial panel
> ([`plugin-framework-review.md`](plugin-framework-review.md)) found 5 blockers in
> v1; this revision closes them and folds in the majors. The maintainer decisions
> from the review are now baked in: **explicit generated registration** (not
> implicit dependency linking), **upgrade to sqlx 0.9** for per-plugin migration
> tables, and **API-typed / navigation-runtime-checked** on the frontend.
> Web-research backing is in [`plugin-framework-research.md`](plugin-framework-research.md)
> (~55 cited sources); design changes it drove are tagged **[research]**.
>
> **Goal:** Let anyone develop, extend, and ship a working end product by adding a
> self-contained **plugin** to the app without hand-editing the central router,
> OpenAPI document, migration list, or frontend route tree — and without losing
> the template's single-binary deploy or its typed **API** contract. (One honest
> caveat survives: enabling a plugin requires adding its crate to the build graph;
> see §2.)

## 1. Why this is hard in *this* template

The template is deliberately explicit and compile-time wired. Today a new
resource must be registered by hand in several central places, and a suite of
guard tests exists precisely because that hand-wiring is error-prone:

- **Flat router** — every route is a literal `.route("/api/v1/...", ...)` call
  in `src/api.rs:59-101`. `routes_and_openapi_spec_are_in_parity`
  (`tests/api.rs:110`) **forbids `.nest(`/`.merge(`** (`tests/api.rs:164-169`)
  because its source-text parser cannot see them.
- **Hand-maintained OpenAPI** — every handler is listed in `paths(...)` and
  every type in `components(schemas(...))` in `src/api.rs:27-53`. Omitting a
  schema is a *silent* failure; `openapi_spec_has_no_dangling_schema_refs`
  (`tests/api.rs:54`) exists only to catch it.
- **The "three-place footgun"** — `AGENTS.md:117-122` calls registering a
  resource in three separate places "the most common mistake."
- **Central frontend route tree** — pages are imported and listed by hand in
  `interface/src/router.tsx:94-116`, with a matching `<NavLink>` in `Layout`.

So a "plugin framework" here is about **making the wiring structural instead of
manual**: a plugin declares what it contributes, and the router, OpenAPI
document, migrations, and UI nav are *built by iterating the set of plugins*.
When the router and the spec are generated from the same declaration, the
three-place footgun and route↔spec drift become **impossible by construction**.

This crosses `AGENTS.md:86` (*"architectural conventions not represented by both
worked examples"*), which is why this is a design doc for review. See §9.

## 2. Chosen approach: A + B combined

> **[implementation correction, 2026-06-23]** As written below this design does
> not compile: the host's generated `src/plugins.rs` referencing each plugin
> crate, while every plugin depends on `app-starter` for the trait/`AppState`,
> forms a Cargo dependency cycle (`app-starter` ⇄ plugin). The implementation
> breaks it by extracting a leaf **`app-starter-plugin-api`** crate (the `Plugin`
> trait + `AppState` + `PLUGIN_API_VERSION`) that both the host and every plugin
> depend on; `app-starter` keeps the registry and re-exports the contract. Plugins
> therefore write `impl app_starter_plugin_api::Plugin`, not `impl
> app_starter::Plugin`. See docs/plugin-framework-impl-status.md (iter-4 blocker).

- **B — Compile-time registration (backend).** A `Plugin` trait; the router and
  OpenAPI document are built by iterating the registered plugins.
  **[review B1] Registration is explicit, not implicit.** `inventory`-style
  "a plugin registers just by being a dependency" is *unsound* under Rust's link
  model: an upstream rlib from which the final binary references no symbol is
  dropped wholesale, constructors and all (confirmed: rust-ctor#27, inventory
  #11/#32/#34/#52/#85). So the host links plugins through a **generated
  `src/plugins.rs`** that names each plugin crate explicitly (`pub use` +
  `register()` call). The scaffolder writes that one line; nothing else central
  changes. This makes registration deterministic across `lto`/`strip`/targets.
- **A — Scaffolding (authoring).** An `add-plugin` skill + `just new-plugin`
  task generate a compliant skeleton (crate + migration + React page + manifest)
  *and* append the crate to the workspace + generated registry, then validate
  against the gates.

**The honest promise.** A plugin author never edits `src/api.rs`, the OpenAPI
document, `interface/src/router.tsx`, or the migration list. They *do* (via the
scaffolder, one line each) add the crate to the workspace `members` and to the
generated `src/plugins.rs`. We do not claim "zero central edits"; we claim
"the scaffolder owns the only central edits, and they are mechanical."

Frontend mirror: build-time discovery via Vite `import.meta.glob` (§3) — the
JS-side equivalent, with its own caveats addressed below.

## 3. Anatomy of a plugin

A plugin is a **Cargo workspace member crate** (backend) with a **co-located
frontend directory** (discovered by the SPA build). A crate is what makes plugins
independently developable and publishable.

```
plugins/
  guestbook/
    plugin.toml                 # manifest: name, version, author, host_api
    Cargo.toml                  # workspace member; depends on app-starter
    src/
      lib.rs                    # domain + handlers + `impl Plugin` + register()
    migrations/
      20260701000001_create_guestbook_entries.sql
    frontend/
      plugin.tsx                # exports a PluginRoute descriptor (glob target)
      Guestbook.tsx
      Guestbook.test.tsx
```

`plugin.toml` (an informational manifest — written by `just new-plugin` but NOT
parsed at runtime; the Rust `impl Plugin` is the enforced source of truth):

```toml
[plugin]
name = "guestbook"                 # the namespace key — route prefix + schema prefix derive from it
version = "0.1.0"
description = "A public guestbook resource"
author = "community@example.com"
host_api = "^1"                    # informational mirror of the impl's host_api(); the impl is what the host checks
frontend = "frontend/plugin.tsx"
```

**[research] Namespace by construction.** Following Kubernetes
(`/apis/<group>/...`, collisions structurally impossible) rather than
WordPress-style prefix discipline: the route prefix (`/api/v1/<name>`), the
OpenAPI **component-name prefix** (`<name>_Item`), and the DB table prefix
(`<name>_*`, §5) are all **derived from `name`** — there is one source of truth,
not a hand-declared `api_prefix`. The component-name prefix is not cosmetic: when
OpenAPI specs are merged, same-named components are **silently last-wins-
overwritten** by naive/`.merge()` composition (the design's own path), so two
plugins each defining `Item`/`Settings` would corrupt the generated client
without it. A guard enforces the derivation (§6).

**`host_api` compatibility.** Mature ecosystems pin plugin↔host compat
with one machine-checked declaration that fails loudly (VS Code `engines.vscode`,
Grafana `grafanaDependency`, go-plugin's protocol version). The core exposes a
`PLUGIN_API_VERSION` constant; each plugin's `host_api()` is a semver range against
it. At startup, `db::validate_registry` (`src/db.rs`, run from `run_all_migrators`
before any migration) refuses a plugin whose `host_api` range does not match this
host's `PLUGIN_API_VERSION`, or is unparseable — and, in the same pass, one whose
`name` is not a safe `^[a-z][a-z0-9_]*$` identifier — with a human-readable error
naming the plugin. A unit test plus the §6 `plugin_names_are_valid_identifiers`
guard keep it honest.

### Backend contract

The core exposes a small trait; each plugin returns its routes and OpenAPI
fragment from **one** source of truth so they cannot drift:

```rust
// core: src/plugin.rs (new)
pub const PLUGIN_API_VERSION: &str = "1.0.0";

pub trait Plugin: Send + Sync + 'static {
    /// Stable identifier; the route/schema/table namespace key.
    fn name(&self) -> &'static str;

    /// Required host-API semver range, checked against PLUGIN_API_VERSION at
    /// startup (plugin.toml's host_api is an informational mirror of this).
    fn host_api(&self) -> &'static str;

    /// Routes AND their OpenAPI paths/schemas, built together so they can't
    /// desync. utoipa-axum's OpenApiRouter registers a handler once and
    /// contributes both the axum route and the OpenAPI path+schemas.
    fn api(&self) -> utoipa_axum::router::OpenApiRouter<AppState>;

    /// Migrations this plugin owns (embedded at compile time, plugin-relative).
    fn migrator(&self) -> Option<sqlx::migrate::Migrator> { None }
}
```

A plugin implements it once and exposes a `register()` the generated module calls:

```rust
// plugins/guestbook/src/lib.rs
struct Guestbook;

impl app_starter::Plugin for Guestbook {
    fn name(&self) -> &'static str { "guestbook" }
    fn host_api(&self) -> &'static str { "^1" }
    fn api(&self) -> OpenApiRouter<AppState> {
        // routes! contributes route + OpenAPI path + schemas in one call
        OpenApiRouter::new().routes(routes!(list_entries, create_entry))
    }
    fn migrator(&self) -> Option<Migrator> { Some(sqlx::migrate!("./migrations")) }
}

/// Called from the host's generated src/plugins.rs (explicit link, [review B1]).
pub fn register() -> Box<dyn app_starter::Plugin> { Box::new(Guestbook) }
```

```rust
// core: src/plugins.rs  (GENERATED by the scaffolder — the only central edit)
pub fn all() -> Vec<Box<dyn crate::Plugin>> {
    vec![
        guestbook::register(),   // one line per plugin, written by `just new-plugin`
        // <scaffolder inserts here>
    ]
}
```

`router()` and the OpenAPI doc iterate `plugins::all()`. Because the host names
each `register()` symbol, the linker pulls the crate in — no silent-empty
registry. Handler authoring is **unchanged** (`AGENTS.md:102-116`): same
domain-module shape, `#[utoipa::path]`, `AppError`. Only registration is
automatic.

**Versions (verified compatible with the repo).** `utoipa-axum = "0.2"` pairs
with `axum 0.8` + `utoipa 5` (MSRV 1.88) — all already in `Cargo.toml`. `utoipa`
keeps its `axum_extras` feature. The `Plugin` trait is object-safe (all methods
take `&self`, return `Self`-free types), and `AppState: Clone+Send+Sync+'static`
makes the per-plugin `OpenApiRouter<AppState>` merge type-check.

### Frontend contract

Each plugin's `frontend/plugin.tsx` exports a `PluginRoute` descriptor (the
contract type lives in `interface/src/plugins/contract.ts`); the SPA discovers
all of them at build time:

```tsx
// plugins/guestbook/frontend/plugin.tsx
import type { PluginRoute } from "../../../interface/src/plugins/contract";
export default {
  path: "/guestbook",
  label: "Guestbook",
  component: () => import("./Guestbook").then((m) => m.Guestbook), // lazy
} satisfies PluginRoute;
```

```tsx
// interface/src/plugins/registry.ts (new)
// Lazy (not eager) so each plugin code-splits into its own chunk.
const modules = import.meta.glob<{ default: PluginRoute }>(
  "../../../plugins/*/frontend/plugin.tsx",
);
```

`router.tsx` builds its children and nav by iterating the discovered routes,
wiring each through TanStack's `route.lazy`.

**[review B4] What is and isn't typed.** The plugin's **API access stays typed
end to end** — pages consume the generated `schema.d.ts` through the openapi-fetch
client exactly as today, and plugin paths/components are namespaced (`<name>_*`)
so the merged `schema.d.ts` stays coherent. But **navigation is not statically
typed**: TanStack derives `<Link to=…>` safety from a *statically known* route
tree, and a runtime-mapped `PluginRoute[]` erases the path literals. We accept
this (per decision) and compensate with a **runtime guard test** asserting every
`pluginRoutes[].path` is unique, well-formed, and resolves. We do **not** claim
compile-time link safety for cross-plugin navigation.

**[review B5] Required central config (scaffolder-owned).**
- `interface/vite.config.ts` must set `server.fs.allow` to include the repo root
  (e.g. `searchForWorkspaceRoot(process.cwd())`), or the **dev server 403s** on
  the sibling `plugins/` dir (there is no JS-workspace marker, so Vite's served
  root is `interface/`). Production `vite build` is unaffected.
- The glob pattern stays **relative** (`../../../plugins/...`); an absolute
  `/plugins` would resolve to the Vite root, not the repo root, and never match.
- **[review M10]** `interface/tsconfig.json` `include` and `biome.json` must be
  extended to cover `../plugins/*/frontend`, or `just lint`'s `tsc --noEmit` +
  Biome would silently skip plugin UI — *reducing* coverage versus today.

## 4. How the central files change

| File | Today | After |
|------|-------|-------|
| `src/plugins.rs` (new, generated) | — | `all()` lists `register()` per plugin — **[B1]** the explicit link that makes the linker include each plugin crate |
| `src/api.rs` | Hand-listed `paths`/`components`/`.route` | `router()` merges `OpenApiRouter`s from `plugins::all()`; core keeps only `/api/health` + `/api/openapi.json` + shared layers |
| `src/db.rs:25` | `migrate!("./migrations").run` | `run_all_migrators(pool)`: core first, then each plugin's migrator into its own `_sqlx_migrations_<name>` table; sets `busy_timeout`+WAL first (§5) |
| `tests/common.rs` | `migrate!("./migrations")` directly | **[B3]** calls the same `run_all_migrators(pool)` so the test DB matches `db::init()` once examples move into plugins |
| `src/bin/openapi_spec.rs` | prints `ApiDoc::openapi()` | **[M2]** builds the spec from the server's own `router()` via `split_for_parts()` so typegen can't diverge from what's served |
| `interface/vite.config.ts` | no `server.fs` | **[B5]** `server.fs.allow` includes repo root |
| `interface/tsconfig.json` / `biome.json` | scope `src` | **[M10]** also include `../plugins/*/frontend` |
| `interface/src/router.tsx` | Hand-listed routes + `<NavLink>`s | Iterate discovered `pluginRoutes` (lazy) to build children + nav |
| `tests/api.rs` | Source-text parity parser; bans `.nest`/`.merge` | Registry-driven parity (§6); `.nest`/`.merge` ban lifted |

Shared HTTP concerns (`CorsLayer`, body limit, timeout, request-id, SPA fallback)
stay in `src/api.rs::router` (`AGENTS.md:202-205`). Plugins contribute *routes*,
never *layers*. **Assembly order is fixed:** merge all plugin + core routers →
`split_for_parts()` → attach SPA fallback and shared layers **last**; a guard
asserts no plugin contributes a fallback.

## 5. Migrations across plugins — **decision: upgrade to sqlx 0.9**

Core migrations stay in `migrations/` (append-only). Each plugin owns
`plugins/<name>/migrations/` with its own embedded `Migrator`. `run_all_migrators`
runs the core migrator first, then each plugin's in `name` order, **each writing
to its own tracking table** `_sqlx_migrations_<name>` via
`Migrator::dangerous_set_table_name(...)` — so every plugin owns an independent
version keyspace.

> **[review B2] This requires sqlx 0.9.** Verified: `dangerous_set_table_name`
> does **not** exist in the pinned `sqlx 0.8.6` — it first ships in **0.9.0**. Per
> decision, we **upgrade to `sqlx 0.9`** as a dedicated, approval-gated phase
> (§7 Phase 0a) with a breaking-change audit (0.8→0.9 changed `set_*` signatures
> and adds `sqlx.toml`). The table name is set in `run_all_migrators` from the
> validated `name()` (not by the plugin), so the §3 sketch's bare `migrate!`
> return is correct — the host applies the table name before `.run()`.

> **Why not one shared `_sqlx_migrations`?** It is a *documented sqlx failure
> mode*, not a tunable risk: `validate_applied_migrations` returns `VersionMissing`
> for any applied row not in the current Migrator's set, so one Migrator per plugin
> against a shared table makes each plugin see the others' rows as missing and the
> **app refuses to start** ([sqlx#1698], [sqlx#3573]). `ignore_missing(true)` masks
> genuine drift too. Per-plugin tables remove the failure entirely.

Rules (drawn from Rails engines, Django apps, WordPress):

1. **Table-name prefixing** — every plugin table is `<name>_<table>`; a guard
   (§6) parses plugin migrations and **fails on any unprefixed table or a clash
   with a core table** — so prefixing is *enforced*, not convention.
2. **Startup safety [review M4]** — `run_all_migrators` sets `PRAGMA busy_timeout`
   and WAL on the pool before migrating; concurrent startup of two instances
   against one file-backed DB is unsupported (documented).
3. **Ordering & dependency direction [review M3]** — core migrates first; a
   plugin may FK only to *already-migrated* core tables; **core never depends on a
   plugin table**. There is no cross-plugin dependency mechanism — plugins must be
   independent (a plugin must not FK another plugin's tables).
4. **Partial failure** — if plugin B's migrator fails after A's committed, startup
   aborts with the failing plugin named; the DB is left with A applied (each table
   is its own keyspace), and re-run resumes. Documented, not hidden.
5. **Uninstall policy** — removing a plugin orphans its `<name>_*` tables and
   `_sqlx_migrations_<name>` by default; `just plugin-purge <name>` drops them.
   (Discourse cites the missing uninstall story as the reason to discourage
   plugin tables; we answer it.)
6. **Append-only per plugin** (`AGENTS.md:177`) — never edit/rename a committed
   plugin migration.

[sqlx#1698]: https://github.com/launchbadge/sqlx/issues/1698
[sqlx#3573]: https://github.com/launchbadge/sqlx/issues/3573

## 6. Guard tests — repurposed and extended

- **`openapi_spec_has_no_dangling_schema_refs`** — kept as-is.
- **`routes_and_openapi_spec_are_in_parity`** — the source-text parser is
  replaced by a registry walk (build router + spec from the same
  `plugins::all()`, assert served route set == spec path set). The **`.nest`/
  `.merge` ban is lifted** — merging is the mechanism. **[review M9]** Direction-3
  (the source-vs-spec shadowing check) is *safe to drop* specifically because
  `utoipa-axum` generates route and OpenAPI path from one declaration, so the
  "route deleted but still in spec, shadowed by a parameterized sibling" class is
  impossible by construction — stated, not silently dropped.
- **[review M2] `typegen_spec_matches_server`** — assert the spec emitted by
  `openapi_spec` equals the spec from the server's `router()`, so the generated
  `schema.d.ts` can't drift from what's served.
- **`every_plugin_route_is_under_its_derived_prefix`** — each plugin's routes
  start with `/api/v1/<name>`.
- **`plugin_names_are_valid_identifiers`** — each plugin `name` matches
  `^[a-z][a-z0-9_]*$` (it derives the route/schema/table names); the host also
  enforces this at startup via `db::validate_registry`.
- **[research] `no_cross_plugin_schema_name_collisions`** — plugin OpenAPI
  component names are prefixed by `name`.
- **[review M3] `plugin_tables_are_prefixed_and_unique`** — every plugin-created
  table matches `<name>_*`, no plugin table equals a core table, and plugin
  `name`s (hence tracking-table names) are unique.
- **[review M1] `expected_plugins_are_registered` — run in RELEASE.** Assert the
  registry contains each expected plugin. Because the silent-loss risk is a
  *release* `lto=thin`+`strip` phenomenon (the shipping profile), this guard runs
  against a **release artifact** in CI (a smoke check hitting `/api/openapi.json`
  on the release binary), on every shipped target — a debug-only assertion would
  miss it. Explicit registration (§2) largely prevents the failure; this catches
  regressions.

The version-uniqueness guard from v1 is dropped (per-plugin keyspaces make it
moot); the table-name guard above replaces the part that still mattered.

## 7. Phased migration plan

Each phase keeps `just verify` green and is independently shippable.

- **Phase 0a — sqlx 0.9 upgrade.** Bump `sqlx` 0.8→0.9, fix `set_*`/API breaks,
  run `just verify` + `cargo deny`. Approval-gated (§9). No plugin behavior yet.
- **Phase 0b — Foundations.** Add `utoipa-axum 0.2`; add `src/plugin.rs` (trait,
  `PLUGIN_API_VERSION`) and an empty generated `src/plugins.rs`; convert the
  package to a workspace with a `plugins/*` glob; introduce
  `run_all_migrators(pool)` and route **both** `db::init()` and `tests/common.rs`
  through it **[B3]**. Confirm `default-run`/Docker `--bin app-starter`/`build.rs`
  paths unchanged (§9). Green because nothing registers yet.
- **Phase 1 — Registry-driven router + typegen.** Rewrite `router()`/`ApiDoc` to
  fold in `plugins::all()`; build the typegen spec from the server router
  **[M2]**; repurpose the parity test (§6). items/posts still wired the old way in
  parallel; re-run `just typegen` and commit `schema.d.ts`.
- **Phase 2 — `items` becomes the first plugin (backend + frontend together).**
  Move `src/items.rs` + `src/api/items.rs` + its migration **and**
  `interface/src/pages/Items.tsx` into `plugins/items/`; add the generated
  `register()` line and the Vite/tsconfig wiring (§3); delete central
  registrations; re-run `just typegen`. Moving both halves together keeps the
  suite green (no orphaned page or un-migrated table).
- **Phase 3 — `posts` becomes the second plugin** (same, backend + frontend
  together). Two worked examples → two worked plugins, satisfying
  `AGENTS.md`'s "represented by both examples" rule for the new convention.
- **Phase 4 — Authoring experience.** `add-plugin` skill + `just new-plugin`
  (scaffolds crate + migration + page + manifest, *and* appends to the workspace +
  generated registry); rewrite the `add-resource` recipe in `AGENTS.md`; add
  `docs/authoring-a-plugin.md`.

A reasonable first PR is **Phases 0a–1** (sqlx upgrade + machinery, no example
moved) so the abstraction is reviewable before migrating a worked example.

## 8. What stays the same (non-goals)

- **No runtime/dynamic loading.** Plugins are compile-time crates; we reject
  WASM/dylib loading for the core (breaks the typed contract, embedded SPA, and
  single-binary deploy; Rust has no stable ABI). **[research]** The escape hatch,
  *if* untrusted or no-rebuild plugins ever become a goal, is the WASM Component
  Model / Extism (in-process, capability-sandboxed) — additive, never native
  dylibs. Research §1.
- **[research/review M7] Trust model, stated plainly.** A plugin crate compiles
  in with full process authority. It can read the SQLite file directly, read
  environment/secrets, and open its own listeners or outbound connections —
  entirely **outside** the CORS/body-limit/timeout layers, which govern only HTTP
  through the shared router and provide **no containment** against in-process code.
  `build.rs`/proc-macros also run arbitrary code at *build* time. There is no
  sandbox and no capability boundary; a manifest here is *disclosure, not
  enforcement*. **"Trusted plugins only" is the sole real control.** Gate any added
  plugin crate through PR review + `cargo-deny` (already in `just verify`);
  `cargo-vet` is recommended once any third-party plugin is added.
- **[review M8] Governance is repo-shaped, not marketplace-shaped.** Because a
  plugin is a compiled-in dependency (no runtime install, no registry, no
  end-user install step), marketplace controls — per-version scanning, "featured"
  tiers, install-count signals — **do not apply**. The actionable controls are PR
  review of the added dependency, `cargo-deny`, and namespace uniqueness (§6).
  Marketplace governance would only become relevant if the WASM escape hatch were
  ever taken.
- **CORS auto-applies.** Because layers wrap the merged router, tightening CORS in
  `src/api.rs` for production covers all plugin routes automatically; plugins
  neither relax nor re-declare it.
- **Handler authoring, versioning, health probe** — unchanged
  (`AGENTS.md:198-205`). **`schema.d.ts` stays generated** (`AGENTS.md:178`).

## 9. Approval boundaries this crosses

Per `AGENTS.md:79-89`, record sign-off for:

1. **Architectural convention not in both examples** (`:86`) — the registry
   pattern; mitigated by Phases 2–3 converting both worked examples.
2. **Major/breaking dependency upgrade** (`:82`) — **sqlx 0.8→0.9** (Phase 0a),
   plus new deps `utoipa-axum` and the workspace conversion.
3. **License policy** (`:87`) — transitive deps of the new crates must satisfy
   `deny.toml`; adopting `cargo-vet` is itself a dependency-policy decision.
4. **Guard-test edits + lifting the `.nest`/`.merge` ban** (`:86`) — §6. (Items 1
   and the ban-lift are one coupled decision with adopting `utoipa-axum`.)
5. **Migration handling** (`:84`) — per-plugin migrators + the 0.9 tracking-table
   API (stays append-only).
6. **Release / Docker / binary name** (`:83`, `:192-193`) — the single-crate →
   workspace conversion must preserve `default-run = "app-starter"`, the Docker
   `--bin app-starter` build, the embedded-SPA `build.rs` paths, and the
   `app-starter`/`app_starter` names the `cut-release`/`setup.sh` flows rely on.
   Phase 0b includes an explicit checklist proving this.

Out of scope / unaffected: auth, public-exposure defaults, `VISION.md`.

## 10. Resolved decisions & remaining questions

**Resolved (this revision):** registration is **explicit/generated** (not
implicit); migrations use **sqlx 0.9 per-plugin tables**; frontend is **API-typed,
navigation-runtime-checked**; first PR is **Phases 0a–1**.

**Still open (smaller, can be deferred):**

1. **API tier** — add a VS Code-style "stable vs proposed" tier for the
   plugin-facing trait now, or defer until there's a second contract consumer? The
   `PLUGIN_API_VERSION` constant (§3) is the seed either way.
2. **`cargo-vet` timing** — adopt audit-before-entry now, or keep
   `cargo-deny`/`cargo-audit` (already gating) and add `cargo-vet` before the
   first third-party plugin lands?
3. **Uninstall default** — orphan-and-offer-purge (§5.5) vs. auto-teardown on
   removal? Orphan is safer for data; auto-teardown is cleaner for dev.

---

*Design only. On approval, implementation proceeds phase by phase, each gated by
`just verify` and the (repurposed) guard tests — including the release-profile
registration smoke check (§6).*
