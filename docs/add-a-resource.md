# Add a resource end to end → see "Authoring a plugin"

> **This recipe is superseded.** Resources are now self-contained **plugins**, so
> you no longer hand-edit the central router, OpenAPI document, migration list, or
> `router.tsx`. The full, current walkthrough lives in
> **[authoring-a-plugin.md](authoring-a-plugin.md)**; the design is in
> [plugin-framework.md](plugin-framework.md).

## TL;DR

```bash
just new-plugin <name>   # scaffold the crate + migration + page + manifest, auto-wired
# ...customize the generated migration / domain / handlers / page...
just typegen             # regenerate interface/src/api/schema.d.ts, then commit it
just verify             # lint + tests (incl. the §6 plugin guards) + frontend + cargo-deny
```

The two worked examples are plugins to copy: `plugins/todo/` (minimal CRUD) and
`plugins/blog/` (status lifecycle, filtered/paginated list, stats). The frontend
page for each lives under `interface/src/plugins/<name>/`.

Use the **`add-plugin`** skill for the step-by-step procedure (it supersedes the
legacy `add-resource` skill). What carries over unchanged from the old recipe:
append-only migrations, handlers returning `Result<_, AppError>` with
`#[utoipa::path]`, typed-client-only data access, and `just typegen` after any
API/schema change. What changed: registration is automatic (the host iterates the
plugin registry), and namespacing — route prefix `/api/v1/<name>`, OpenAPI
component prefix `<name>_*`, table prefix `<name>_*` — derives from the plugin name
and is enforced by the §6 guard tests in `tests/plugins.rs`.
