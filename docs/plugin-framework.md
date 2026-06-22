# Plugin Framework — Design Doc

> **Status:** Proposal (design only — no code changes yet). Awaiting sign-off
> before implementation. This document is the deliverable for the
> `claude/modular-plugin-framework` branch.
>
> **Goal:** Let anyone develop, extend, and ship a working end product by
> dropping a self-contained **plugin** into the app — without hand-editing the
> central router, OpenAPI document, migration list, or frontend route tree, and
> without losing the template's end-to-end type safety or single-binary deploy.

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
  `interface/src/router.tsx:94-116`, with a matching `<NavLink>` in `Layout`
  (`interface/src/router.tsx:73-75`).

So a "plugin framework" here is really about **making the wiring structural
instead of manual**: a plugin declares what it contributes, and the router,
OpenAPI document, migrations, and UI nav are *built by iterating the set of
plugins*. When the router and the spec are generated from the same declaration,
the three-place footgun and route↔spec drift become **impossible by
construction** rather than caught after the fact.

This crosses an explicit approval boundary in `AGENTS.md:86` —
*"architectural conventions not represented by both worked examples"* — which is
why this is a design doc for review, not a direct change. See §9.

## 2. Chosen approach: A + B combined

Per the decision on this branch:

- **B — Compile-time auto-registration (backend).** A `Plugin` trait; each
  plugin registers itself via the [`inventory`] crate; the router and OpenAPI
  document are built by iterating the registry. Full type safety, single binary,
  no central edits. Frontend mirror: build-time discovery via Vite's
  `import.meta.glob` (the JS equivalent of auto-registration — no codegen step).
- **A — Scaffolding (authoring).** An `add-plugin` skill + `just new-plugin`
  task generate a compliant plugin skeleton (crate + migration + React page +
  manifest) and validate it against the same gates. This is the "anyone can
  start" on-ramp.

The result: a contributor runs one command, fills in domain logic, and the
plugin wires itself in. They never touch `src/api.rs` or `router.tsx`.

[`inventory`]: https://docs.rs/inventory

## 3. Anatomy of a plugin

A plugin is a **Cargo workspace member crate** (backend) with a **co-located
frontend directory** (discovered by the SPA build). Making it a crate is what
makes plugins independently developable and publishable — a community author
ships a crate, the app depends on it, and `inventory` links it in.

```
plugins/
  guestbook/
    plugin.toml                 # manifest: name, version, author, description
    Cargo.toml                  # workspace member; depends on app-starter-core
    src/
      lib.rs                    # domain + handlers + `impl Plugin` + register!
    migrations/
      20260701000001_create_guestbook.sql
    frontend/
      plugin.tsx                # exports routes + nav entry (glob target)
      Guestbook.tsx
      Guestbook.test.tsx
```

`plugin.toml` (manifest, consumed by the scaffolder and `just plugins`):

```toml
[plugin]
name = "guestbook"
version = "0.1.0"
description = "A public guestbook resource"
author = "community@example.com"
api_prefix = "/api/v1/guestbook"   # namespace this plugin owns
frontend = "frontend/plugin.tsx"
```

### Backend contract

The core exposes a small trait. Each plugin returns its routes and OpenAPI
fragment from **one** source of truth, so they cannot drift:

```rust
// core: src/plugin.rs (new)
pub trait Plugin: Send + Sync + 'static {
    /// Stable identifier, also the migration/namespace key.
    fn name(&self) -> &'static str;

    /// Routes AND their OpenAPI paths, built together so they can't desync.
    /// utoipa-axum's OpenApiRouter registers a handler once and contributes
    /// both the axum route and the OpenAPI path+schemas in a single call.
    fn api(&self) -> utoipa_axum::router::OpenApiRouter<AppState>;

    /// Migrations this plugin owns (embedded at compile time).
    fn migrator(&self) -> Option<sqlx::migrate::Migrator> { None }
}

/// Registration slot. `inventory` collects every `register_plugin!` across all
/// linked crates with zero central edits.
pub struct PluginRegistration(pub fn() -> Box<dyn Plugin>);
inventory::collect!(PluginRegistration);

#[macro_export]
macro_rules! register_plugin {
    ($ctor:expr) => {
        inventory::submit! { $crate::plugin::PluginRegistration($ctor) }
    };
}
```

A plugin implements it once:

```rust
// plugins/guestbook/src/lib.rs
struct Guestbook;

impl Plugin for Guestbook {
    fn name(&self) -> &'static str { "guestbook" }

    fn api(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
            // one registration → route + OpenAPI path + schemas
            .routes(routes!(list_entries, create_entry))
    }

    fn migrator(&self) -> Option<Migrator> {
        Some(sqlx::migrate!("./migrations"))
    }
}

app_starter::register_plugin!(|| Box::new(Guestbook));
```

The domain module, handlers, and `#[utoipa::path]` annotations are written
**exactly as the `items`/`posts` recipe describes today** (`AGENTS.md:102-116`).
Nothing about writing a handler changes — only the registration becomes
automatic. `utoipa_axum::routes!` is what collapses the "register the route" and
"list it in `paths(...)` + `components(schemas(...))`" steps into one, killing
the three-place footgun.

### Frontend contract

Each plugin's `frontend/plugin.tsx` exports a descriptor; the SPA discovers all
of them at build time with `import.meta.glob` — no central edit, no codegen:

```tsx
// plugins/guestbook/frontend/plugin.tsx
import { Guestbook } from "./Guestbook";
export default {
  path: "/guestbook",
  label: "Guestbook",
  component: Guestbook,
} satisfies PluginRoute;
```

```tsx
// interface/src/plugins/registry.ts (new, ~15 lines)
const modules = import.meta.glob<{ default: PluginRoute }>(
  "../../../plugins/*/frontend/plugin.tsx",
  { eager: true },
);
export const pluginRoutes: PluginRoute[] = Object.values(modules)
  .map((m) => m.default)
  .sort((a, b) => a.label.localeCompare(b.label));
```

`router.tsx` then builds its children and nav by iterating `pluginRoutes`
instead of listing each page by hand. The pages themselves are written exactly
as `Items.tsx`/`Posts.tsx` are today (Radix Themes + sections + typed hooks);
they still consume the generated `schema.d.ts`, so the plugin's API stays typed
end to end.

## 4. How the central files change

| File | Today | After |
|------|-------|-------|
| `src/api.rs` | Hand-listed `paths(...)`, `components(schemas(...))`, and `.route(...)` calls | `router()` merges `OpenApiRouter`s from `inventory::iter::<PluginRegistration>`; core keeps only `/api/health` + `/api/openapi.json` and the shared layers |
| `src/db.rs:25` | `sqlx::migrate!("./migrations").run(...)` | Run core migrator, then each plugin's `migrator()` in deterministic order |
| `interface/src/router.tsx` | Hand-listed routes + `<NavLink>`s | Iterate `pluginRoutes` to build children + nav |
| `tests/api.rs` | Source-text parity parser; bans `.nest`/`.merge` | Registry-driven parity (see §6); `.nest`/`.merge` ban lifted |

The shared HTTP concerns (`CorsLayer`, body limit, timeout, request-id, SPA
fallback) stay in `src/api.rs::router` exactly as `AGENTS.md:202-205` requires —
plugins contribute *routes*, never *layers*.

## 5. Migrations across plugins

Core migrations stay in `migrations/` (append-only, unchanged). Each plugin
owns `plugins/<name>/migrations/` with its own embedded `Migrator`. At startup
`db::init()` runs the core migrator, then each plugin's, in name order.

**The one real hazard:** sqlx records every migration in a single shared
`_sqlx_migrations` table keyed by the `i64` version (the timestamp). Two plugins
that pick the *same* timestamp collide. Mitigations:

1. The scaffolder always stamps a unique current-time version.
2. A new guard (§6) fails the build if any two migrations across core + all
   plugins share a version.
3. Per-plugin append-only rule, identical to the core rule
   (`AGENTS.md:177`): never edit or rename a committed plugin migration.

> **Open question for sign-off:** an alternative is a per-plugin migration
> *table* (`_sqlx_migrations_<plugin>`), which fully isolates plugins but needs a
> custom migrator wrapper since sqlx hard-codes the table name. The shared-table
> + collision-guard approach above is simpler and is the recommended default.

## 6. Guard tests — repurposed, not removed

The existing guards encode real invariants; the framework makes most of them
*structural*, but we keep equivalents as defense in depth:

- **`openapi_spec_has_no_dangling_schema_refs`** — kept as-is. Still valid and
  cheap.
- **`routes_and_openapi_spec_are_in_parity`** — the source-text parser
  (`tests/api.rs:159-175`, `declared_v1_route_paths`) is **replaced** by a
  registry walk: build the router and the spec from the same
  `inventory::iter`, then assert the served route set equals the spec path set.
  With `utoipa-axum` the two are generated from one declaration, so this becomes
  a regression guard rather than a footgun-catcher. The **`.nest`/`.merge` ban
  (`tests/api.rs:164-169`) is lifted** — merging is now the mechanism.
- **New: `plugin_migration_versions_are_unique`** — scans core + every plugin
  migration dir and fails on a duplicate timestamp (§5).
- **New: `every_plugin_route_is_under_its_declared_prefix`** — each plugin's
  routes must live under the `api_prefix` in its `plugin.toml`, so two plugins
  can't claim the same path.

Editing these guard tests and lifting the `.nest`/`.merge` ban is itself an
architectural-convention change requiring sign-off (§9).

## 7. Phased migration plan

Each phase is independently shippable and keeps `just verify` green.

- **Phase 0 — Foundations (no behavior change).** Add `inventory` and
  `utoipa-axum` deps. Add `src/plugin.rs` (trait, registration macro, registry
  iterator). Convert the workspace to include a `plugins/` glob. Gates stay
  green because nothing registers yet.
- **Phase 1 — Registry-driven router.** Rewrite `src/api.rs::router` and
  `ApiDoc` to fold in `OpenApiRouter`s from the registry, with core endpoints
  still hard-wired. Repurpose the parity test (§6). At this point the registry
  is empty of resources — items/posts still wired the old way behind a temporary
  shim — so this phase proves the merge + parity machinery in isolation.
- **Phase 2 — `items` becomes the first plugin.** Move `src/items.rs` +
  `src/api/items.rs` + its migration into `plugins/items/`. Delete its central
  registrations. This is the canonical backend worked example.
- **Phase 3 — `posts` becomes the second plugin.** Same for the richer
  lifecycle example. Two worked examples → two worked plugins, preserving
  `AGENTS.md`'s "represented by both examples" rule for the *new* convention.
- **Phase 4 — Frontend discovery.** Add `interface/src/plugins/registry.ts`
  (`import.meta.glob`); move `Items.tsx`/`Posts.tsx` into their plugin
  `frontend/` dirs; rewrite `router.tsx` to iterate `pluginRoutes`.
- **Phase 5 — Migration discovery + guards.** Per-plugin migrators in
  `db::init()`; add the two new guard tests.
- **Phase 6 — Authoring experience.** `add-plugin` skill + `just new-plugin`
  scaffolder; rewrite the `add-resource` recipe in `AGENTS.md` to point at the
  plugin flow; add a human-facing `docs/authoring-a-plugin.md`.

A reasonable first PR is **Phases 0–1** (machinery, no resource moved) so the
core abstraction can be reviewed before any worked example is migrated.

## 8. What stays the same (non-goals)

- **No runtime/dynamic loading.** Plugins are compile-time crates. We
  explicitly reject WASM/dylib loading (option C from the discussion): it breaks
  the typed OpenAPI contract, the embedded-SPA model, and single-binary deploy.
- **Handler authoring is unchanged.** Same domain-module shape, same
  `#[utoipa::path]`, same `AppError`, same typed hooks. Only *registration*
  becomes automatic.
- **Versioning, CORS, body limits, request-id, health probe** — unchanged and
  still centrally owned (`AGENTS.md:198-205`).
- **`schema.d.ts` stays generated.** `just typegen` still produces it from the
  (now registry-built) OpenAPI doc; never hand-edited (`AGENTS.md:178`).

## 9. Approval boundaries this crosses

Per `AGENTS.md:79-89`, the following need explicit human sign-off and should be
recorded against this PR/issue before implementation:

1. **Architectural convention not represented by both examples**
   (`AGENTS.md:86`) — the entire registry pattern. Mitigated by Phases 2–3
   converting *both* worked examples so the new convention is doubly represented.
2. **New dependencies** (`AGENTS.md:82`) — `inventory` and `utoipa-axum` (and
   the cargo-deny license check that `just verify` runs must pass).
3. **Editing the guard tests** and lifting the `.nest`/`.merge` ban
   (`AGENTS.md:86`) — see §6.
4. **Migration handling** (`AGENTS.md:84`) — per-plugin migrators touch how
   migration history is applied (though it stays append-only).

Out of scope / unaffected: auth, public-exposure defaults, release workflow,
Docker, `VISION.md` (referenced for intent, never edited).

## 10. Open questions for reviewers

1. **Plugin migration isolation** — shared `_sqlx_migrations` + collision guard
   (recommended) vs. per-plugin table (§5)?
2. **Plugin enable/disable** — implicit (a plugin is "on" if its crate is a
   dependency) vs. an explicit Cargo feature per plugin for opt-out builds?
3. **First PR scope** — machinery-only (Phases 0–1) for review, or land through
   Phase 3 so the worked examples prove it end to end?
4. **`utoipa-axum` adoption** — it is the lever that removes the dual
   path/schema registration; acceptable as a core dependency?

---

*Design only. On approval, implementation proceeds phase by phase, each phase
gated by `just verify` and the (repurposed) guard tests.*
