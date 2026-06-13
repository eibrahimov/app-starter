## What and why

<!-- One or two sentences: the problem and the change. Why is this good for App Starter, not just interesting? -->

## Template / generated-project impact

<!-- Does this affect generated projects, defaults, docs, examples, or release behavior? If no, say no. -->

## Validation

- [ ] `just lint`
- [ ] `just test`
- [ ] `just check-typegen` (`just typegen` run and `schema.d.ts` committed if the API changed)
- [ ] Other relevant gates: <!-- just build / just docker-build / just desktop-build -->
- Skipped gates and reason:

## Approval / risk notes

- Required approval or discussion links:
- Migration files added:
- Generated files updated:
- Docs / AGENTS / README updates needed:
- Residual risks:

## Checklist

- [ ] This keeps the one-binary + OpenAPI-source-of-truth identity intact
- [ ] Migrations are new files only (none edited or renamed)
- [ ] `interface/src/api/schema.d.ts` was regenerated, not hand-edited
- [ ] Defaults touching security, CORS, dependencies, release, migrations, generated files, or architecture have linked approval
- [ ] README / AGENTS.md / docs updated if conventions or generated-project behavior changed
- [ ] No license-encumbered code or third-party product names
- [ ] Conventional commit title (`feat:`, `fix:`, `ci:`, `docs:`, `test:`)
