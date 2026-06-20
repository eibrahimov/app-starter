---
name: add-resource
description: >-
  Add a new REST resource end to end to this app-starter template: migration ->
  domain module -> API handlers -> OpenAPI registration -> tests -> typegen ->
  frontend page + route, following the items/posts worked examples. Use when asked
  to add a resource, add an endpoint, scaffold CRUD, wire a new table/model through
  to the UI, or "do this like items/posts". Encodes the invariants that the prose
  docs describe and ends with a validation script that catches the silent
  OpenAPI-registration footgun.
---

# Add a resource end to end

This skill is the **operational procedure** for adding a resource. The
copy/paste-level detail lives in two canonical docs — keep this skill in sync with
them rather than restating everything:

- [`AGENTS.md`](../../../AGENTS.md) -> "Adding a resource end to end" (agent-facing)
- [`docs/add-a-resource.md`](../../../docs/add-a-resource.md) (human-facing, with snippets)

Copy the shape of the two worked examples exactly: `items` (minimal CRUD) and
`posts` (status lifecycle + filtered/paginated list + get-by-id + stats).

## Read these invariants first (this is where resources break)

1. **The three-place registration footgun (most common mistake).** In
   `src/api/mod.rs` you must register a new endpoint in THREE places:
   (a) `pub mod <resource>;`, (b) the handler in `#[openapi(paths(...))]` **and
   every new `ToSchema` type in `components(schemas(...))`**, (c) a
   `.route("/api/v1/...", ...)` placed **before** `.fallback(crate::frontend::spa)`
   (axum 0.8 brace syntax: `{id}`). Omitting a type from `components(schemas(...))`
   does **not** error — the endpoint silently vanishes from the OpenAPI spec and the
   generated TypeScript. The `openapi_spec_has_no_dangling_schema_refs` guard test
   catches this; the validation script runs it.
2. **Migrations are append-only.** New file `migrations/<YYYYMMDDHHMMSS>_<desc>.sql`
   whose timestamp sorts after every existing migration. NEVER edit or rename a
   committed migration — sqlx checksums them and the app refuses to start otherwise.
3. **Every `#[utoipa::path(...)]` carries the FULL literal path** including the
   `/api/v1` prefix, a per-resource `tag`, `params`, `request_body`, and every
   response status with `body =` schema.
4. **Never hand-edit `interface/src/api/schema.d.ts`.** It is generated — run
   `just typegen` and commit the result.
5. **`/api/v1` is additive-only.** Within v1, only ADD fields/endpoints; never remove
   or repurpose them. A breaking change opens `/api/v2` alongside v1.
6. **Frontend rules:** data access only through the typed `api` client from
   `../api/client` (never raw `fetch`); `useQuery` array keys; mutations
   `invalidateQueries` on success; explicit isLoading/isError branches; styling
   via Radix Themes component props (no Tailwind); spaces not tabs; no `@/` path aliases.

## Procedure

Work in this order; each step maps to the canonical docs for snippets.

1. **Migration** — `migrations/<timestamp>_<desc>.sql`. TEXT uuid PK generated
   app-side, INTEGER booleans, TEXT timestamps, index on sort columns.
2. **Domain module** — `src/<resource>.rs`: row struct deriving
   `Debug, Serialize, sqlx::FromRow, utoipa::ToSchema`; payload structs deriving
   `Debug, Deserialize, ToSchema`; plain `async fn(pool: &SqlitePool, ...) ->
   Result<_, sqlx::Error>` queries. By-id mutations return `Result<bool, _>` via
   `rows_affected()`. No repository structs/traits. Unit-test pure logic in a
   `#[cfg(test)] mod tests` (see `src/posts.rs`).
3. **Export** — add `pub mod <resource>;` to `src/lib.rs`. Extend `AppState` only for
   genuinely shared values (the pool usually suffices — extending it needs approval,
   see below).
4. **Handlers** — `src/api/<resource>.rs`: thin `async fn` taking
   `State(state): State<AppState>`, returning `Result<_, AppError>`. Validate input
   (`AppError::BadRequest`); map `false`/`None` store results to `AppError::NotFound`;
   201 + Json for create, `StatusCode::NO_CONTENT` for delete-like actions. Each
   handler gets a complete `#[utoipa::path(...)]` (invariant 3).
5. **Register in three places** in `src/api/mod.rs` (invariant 1). This is the step
   that silently breaks typegen — double-check `components(schemas(...))`.
6. **Backend tests** — extend `tests/api.rs` reusing `test_app()` / `body_json()`;
   import the crate as `app_starter::...`. Cover the happy-path roundtrip plus one
   400 and one 404.
7. **Typegen** — run `just typegen` and COMMIT the regenerated
   `interface/src/api/schema.d.ts`. Never hand-edit it. Skipping this fails `just lint`
   (tsc) and the typegen-drift CI job.
8. **Frontend page** — `interface/src/pages/<Name>.tsx` modeled on `Items.tsx`
   (invariant 6).
9. **Route + nav** — in `interface/src/router.tsx` add a `createRoute({...})`, append
   it to `rootRoute.addChildren([...])`, and add a nav `<Link>` in `Layout`.

## Validate before handoff (mandatory)

Run the bundled validation script from the repo root — it checks the typegen drift
gate and runs the backend tests including the dangling-`$ref` guard:

```
.claude/skills/add-resource/scripts/validate-resource.sh
```

Then run the full CI set before declaring done:

```
just verify
```

Report the commands you ran and their results. If a gate is red, fix it — do not hand
off a red resource. If a command cannot run, state why and the remaining risk
(per `AGENTS.md`).

## Sub-agent review (subjective gate)

The validation script and `just verify` cover the **deterministic** gates; they cannot judge
whether the new code matches the template's conventions. After the gates are green, spawn a
fresh-context review sub-agent (Task tool, `general-purpose`) that did NOT see this skill, give
it the diff (`git diff` plus the new files) and the two worked examples as references, and ask
it to confirm:

- every handler in `src/api/<resource>.rs` carries a complete `#[utoipa::path(...)]` (full
  `/api/v1` path, `tag`, `params`, `request_body`, every response with `body =`);
- every new `ToSchema` type appears in BOTH `paths(...)` and `components(schemas(...))` in
  `src/api/mod.rs` (the silent footgun);
- the domain module avoids repository structs/traits and mirrors `src/posts.rs`;
- `interface/src/pages/<Name>.tsx` mirrors `Items.tsx` (typed `api` client only, array query
  keys, `invalidateQueries` on mutate, explicit isLoading/isError, Radix Themes styling, no `@/` alias);
- the migration is append-only with a last-sorting timestamp.

Incorporate its findings before handoff. This catches convention drift the scripts can't.

## Stop and get human approval before

These exceed the items/posts pattern and are gated by `AGENTS.md` "Approval
boundaries" / `docs/template-direction.md`:

- changing security posture, auth, authorization, CORS, request limits, or public
  exposure;
- extending shared `AppState` with new infrastructure (clients, caches, config);
- introducing an architectural convention not shown by both worked examples
  (e.g. repository traits, a service layer, a CRUD generator);
- anything touching migration history, generated-file policy, or `/api/v1` removal.

Record the approving issue/PR in the change description.
