---
name: add-plugin
description: >-
  Add a new resource to this app-starter template as a self-contained PLUGIN
  (backend crate + co-located frontend page) end to end: scaffold with
  `just new-plugin` -> customize migration/domain/handlers/page -> typegen ->
  verify. Use when asked to add a resource, add an endpoint, scaffold CRUD, wire a
  new table/model through to the UI, add a plugin, or "do this like todo/blog".
  The host builds the router, OpenAPI document, migrations, and nav by iterating
  registered plugins, so you never hand-edit the central router/spec/router.tsx.
  Supersedes the legacy `add-resource` skill (central-registration flow).
---

# Add a resource as a plugin

This skill is the **operational procedure**. The reference detail lives in two
canonical docs — keep this skill in sync with them rather than restating
everything:

- [`docs/authoring-a-plugin.md`](../../../docs/authoring-a-plugin.md) (human-facing walkthrough)
- [`docs/plugin-framework.md`](../../../docs/plugin-framework.md) (design / rationale)
- [`AGENTS.md`](../../../AGENTS.md) -> "Adding a resource end to end" (agent-facing summary)

The two worked-example plugins, `plugins/todo/` (minimal CRUD) and `plugins/blog/`
(status lifecycle + filters + stats), are the shapes to copy.

## Procedure

1. **Scaffold.** Run `just new-plugin <name>` (lowercase, `^[a-z][a-z0-9_]*$`,
   e.g. `guestbook`). This generates the backend crate `plugins/<name>/`
   (`Cargo.toml`, `plugin.toml`, `src/lib.rs`, a prefixed migration), the frontend
   page `interface/src/plugins/<name>/`, and makes the two central edits — the path
   dep in the root `Cargo.toml` and the `<name>::register()` line in the generated
   `src/plugins/mod.rs`. (If scaffolding by hand, copy `plugins/todo/` and make
   those two edits yourself.)

2. **Customize the backend** (`plugins/<name>/`):
   - Migration: replace the generated columns with your schema. Keep the
     `<name>_` table prefix. Append-only — never edit a committed migration.
   - Domain + handlers in `src/lib.rs`: keep routes under `/api/v1/<name>`; prefix
     every `ToSchema` component with `#[schema(as = <name>_Type)]`; handlers take
     `State(state): State<AppState>`, return `Result<_, AppError>`, validate input
     (`AppError::BadRequest`), map a missing row to `AppError::NotFound`.
   - Optional demo data: implement the `seed()` hook (idempotent) as todo/blog do.

3. **Customize the frontend** (`interface/src/plugins/<name>/`): build the page
   from Radix Themes + the `sections/` layer + the typed hooks
   (`useApiQuery`/`useApiMutation`), exactly like `Todo.tsx`/`Blog.tsx`. Data
   access only through the typed `api` client (never raw `fetch`). The `add-component`
   skill + [docs/radix-reference.md](../../../docs/radix-reference.md) cover the
   component catalog.

4. **Typegen.** `just typegen`, then COMMIT the regenerated
   `interface/src/api/schema.d.ts` (never hand-edit it).

5. **Verify.** `just verify` must be green. It runs the §6 namespacing guard tests
   (`tests/plugins.rs`): routes under the derived prefix, components prefixed and
   collision-free, tables prefixed and unique, and the plugin registered. CI also
   runs a release-profile registration smoke (`just release-smoke`).

## Invariants (the guards enforce these)

- Namespacing derives from `name`: route prefix `/api/v1/<name>`, component prefix
  `<name>_*`, table prefix `<name>_*`.
- Plugins contribute routes, never layers — CORS / body-limit / timeout /
  request-id / SPA-fallback stay centrally owned in `src/api.rs::router`.
- A plugin depends on `app-starter-plugin-api` (the contract crate), never on
  `app-starter` (that would be a dependency cycle).
- A plugin may FK only to already-migrated core tables; core never depends on a
  plugin table.
- Trusted plugins only — a plugin compiles in with full process authority and is
  not sandboxed (design doc §8). Use bun/bunx for any JS tooling.
