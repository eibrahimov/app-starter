# App Starter

Full-stack Rust starter: one axum binary serves the JSON API and the embedded React SPA,
typed end to end from the backend OpenAPI contract. SQLite via sqlx; Bun + `just` for tooling;
optional Tauri 2 desktop shell. Two worked examples (`items`, `posts`) are wired through every
layer — copy their shape, then replace them with your domain.

See AGENTS.md for project conventions, validation commands, and the resource recipe.

Quick reminders:
- Routine gates: `just lint`, `just test`, `just check-typegen`.
- UI layer is Radix Themes (`@radix-ui/themes`); global config in `interface/src/theme/theme.config.ts`,
  full vocabulary in `docs/radix-reference.md`.
- Run `just typegen` after API/OpenAPI changes; never hand-edit `interface/src/api/schema.d.ts` (generated).
- Never edit or rename committed migrations.
- Use bun/bunx only for JS tooling.
- Do not run `scripts/setup.sh` except for one-time fresh-template initialization.
- `VISION.md` is human-maintained; reference it for intent and never edit it (agents are denied write access).

Skills (`.claude/skills/`) automate common workflows:
- `add-resource` — new REST resource end to end (migration → API → typegen → UI).
- `add-migration` — evolve an existing table's schema (append-only).
- `configure-theme` — restyle the UI from a natural-language request (edits `theme.config.ts`).
- `harden-for-production` — work the production-readiness checklist before public exposure.
- `cut-release` — preflight and tag a `vX.Y.Z` release.
