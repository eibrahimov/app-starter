# Authoring a plugin

A **plugin** is a self-contained resource — backend crate + co-located frontend
page — that the host discovers and wires automatically. You never hand-edit the
central router, OpenAPI document, migration list, or `router.tsx`; the host builds
all of those by iterating the registered plugins. This is the human-facing
companion to the design ([`plugin-framework.md`](plugin-framework.md)) and the
`add-plugin` skill.

Two worked examples ship as plugins — copy their shape:

- **`plugins/todo/`** — minimal CRUD (list/create/toggle/delete).
- **`plugins/blog/`** — a draft → published → archived lifecycle, filtered +
  paginated list, and an aggregate stats endpoint.

## TL;DR

```sh
just new-plugin guestbook   # scaffold crate + migration + page + manifest, wired in
# ...edit the generated migration / domain / handlers / page for your schema...
just typegen                # regenerate interface/src/api/schema.d.ts, then commit it
just verify                 # lint + tests (incl. the §6 guards) + frontend + cargo-deny
```

## Anatomy of a plugin

```
plugins/<name>/                                  backend crate (Cargo workspace member)
  Cargo.toml                                     depends on app-starter-plugin-api, NOT app-starter
  plugin.toml                                    manifest: name, version, host_api, frontend
  src/lib.rs                                     domain + handlers + `impl Plugin` + `register()`
  migrations/<ts>_create_<name>_items.sql        plugin-owned; table prefixed `<name>_*`
interface/src/plugins/<name>/                    frontend (under interface/src so it
  plugin.tsx                                     resolves shared deps + the typed client)
  <Type>.tsx                                     the page (discovered by registry.ts)
```

The namespace **derives from `name`**: route prefix (`/api/v1/<name>`), OpenAPI
component prefix (`<name>_*`), and table prefix (`<name>_*`) all come from it.

## What the scaffolder does

`just new-plugin <name>` (name matches `^[a-z][a-z0-9_]*$`) generates everything
above **and** makes the only two central edits — both mechanical:

1. appends `<name> = { path = "plugins/<name>" }` to the root `Cargo.toml`
   `[dependencies]` (this explicit link is what makes the linker include the
   crate — `inventory`-style implicit registration is unsound under Rust's link
   model, see the design doc §2);
2. appends `<name>::register(),` to the generated `src/plugins.rs`.

It does **not** touch `src/api.rs`, the OpenAPI document, or `router.tsx` — those
are built from the registry.

## The backend contract

`src/lib.rs` implements [`app_starter_plugin_api::Plugin`](../plugin-api/src/lib.rs):

| method | purpose |
|--------|---------|
| `name() -> &'static str` | the namespace key (route/schema/table prefix) |
| `host_api() -> &'static str` | semver range against `PLUGIN_API_VERSION` |
| `api() -> OpenApiRouter<AppState>` | routes **and** their OpenAPI paths/schemas, from one `routes!(...)` declaration so they cannot drift |
| `migrator() -> Option<Migrator>` | `Some(sqlx::migrate!("./migrations"))`; the host applies the `_sqlx_migrations_<name>` table name |
| `seed(pool) -> SeedFuture` | optional, idempotent demo data (default no-op) |

Handlers are unchanged from ordinary axum + utoipa: `State(state): State<AppState>`,
return `Result<_, AppError>`, `#[utoipa::path(...)]` with the full `/api/v1/<name>`
path. **Prefix every `ToSchema` type's component name** with
`#[schema(as = <name>_Type)]` (e.g. `#[schema(as = todo_Todo)]`) — the
`no_cross_plugin_schema_name_collisions` guard enforces it.

## Migrations

The plugin owns `plugins/<name>/migrations/`, embedded with `sqlx::migrate!`, and
runs into its **own** `_sqlx_migrations_<name>` tracking table (an independent
version keyspace). Rules:

- every table is prefixed `<name>_*` (guard-enforced);
- append-only — never edit or rename a committed migration;
- a plugin may FK only to already-migrated **core** tables; a plugin must not FK
  another plugin's tables, and core never depends on a plugin table.

## The frontend

`interface/src/plugins/<name>/plugin.tsx` default-exports a `PluginRoute`
(`{ path, label, component }`, the page loaded lazily so it code-splits). The SPA
discovers it via `interface/src/plugins/registry.ts`; `router.tsx` builds the route
tree + nav from the discovered set. The page lives under `interface/src/` (not the
plugin crate) so it resolves the shared Radix/hook deps and the typed `api` client
normally; data access is typed end to end through `schema.d.ts`. Navigation is not
statically typed (a runtime-mapped route list) — `registry.test.ts` guards that
every path is unique and well-formed.

## The guards (what "done" means)

`just verify` runs, among others, the §6 namespacing guards in `tests/plugins.rs`:

- `every_plugin_route_is_under_its_derived_prefix`
- `plugin_names_are_valid_identifiers` (each `name` matches `^[a-z][a-z0-9_]*$`,
  also host-enforced at startup via `db::validate_registry`)
- `no_cross_plugin_schema_name_collisions` (components prefixed + unique)
- `plugin_tables_are_prefixed_and_unique`
- `expected_plugins_are_registered` (plus a **release-profile** CI smoke,
  `just release-smoke`, since silent registry loss is an `lto`+`strip` phenomenon)

A scaffolded plugin passes all of them out of the box; keep them green as you edit.

## Trust model

A plugin compiles in with full process authority — it is **not** sandboxed. Only
add plugin crates you trust (PR review + `cargo-deny`, both already gating). See
the design doc §8.

## Removing a plugin

Delete `plugins/<name>/` and `interface/src/plugins/<name>/`, remove its
`Cargo.toml` dep line and its `register()` line from `src/plugins.rs`, then
`just typegen`. Its `<name>_*` tables and `_sqlx_migrations_<name>` are orphaned
(left in place for data safety); drop them manually if you want them gone.
