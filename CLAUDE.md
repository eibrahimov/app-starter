See AGENTS.md for project conventions, validation commands, and the resource recipe.

Quick reminders:
- Routine gates: `just lint`, `just test`, `just check-typegen`.
- Run `just typegen` after API/OpenAPI changes; never hand-edit `interface/src/api/schema.d.ts`.
- Never edit or rename committed migrations.
- Use bun/bunx only for JS tooling.
- Do not run `scripts/setup.sh` except for one-time fresh-template initialization.
