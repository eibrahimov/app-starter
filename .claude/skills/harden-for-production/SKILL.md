---
name: harden-for-production
description: >-
  Work the App Starter production-readiness checklist before exposing a generated
  app to real users: CORS/public-exposure, auth, request limits, SQLite data path
  and backups, secrets, observability, API contract, desktop sidecar, and Docker/
  release. Use when asked to harden, prepare for production, go live, do a security or
  pre-launch review, tighten CORS, or "is this safe to deploy?". Runs a read-only
  audit script and anchors every decision to docs/production-readiness.md.
---

# Harden for production

App Starter is production-minded, not production-complete. This skill is the
**operational procedure** for the deliberate decisions in the canonical doc — read it,
do not restate it:

- [`docs/production-readiness.md`](../../../docs/production-readiness.md) (the full checklist)
- [`AGENTS.md`](../../../AGENTS.md) -> "Approval boundaries" (what needs human sign-off)

## The footgun this skill exists to catch

`src/api.rs` ships `CorsLayer::permissive()` so the optional Tauri sidecar can reach
the API from `tauri://localhost`. Permissive CORS on a publicly-exposed API is the single
most common way a generated app ships unsafe. Before public exposure you must either
restrict origins or **explicitly accept** the permissive default for your exposure model —
and document the choice. Do NOT silently remove the Tauri-aware base URL logic in
`interface/src/api/client.ts`; the desktop shell needs it.

## Procedure

1. **Run the audit** (read-only) from the repo root to see the current posture:
   ```sh
   .claude/skills/harden-for-production/scripts/production-audit.sh
   ```
   It flags committed secrets, the permissive CORS default, a placeholder Tauri bundle id,
   and the Cargo.toml version. Treat WARN lines as decisions to make, FAIL lines as blockers.
2. **Walk the checklist** in `docs/production-readiness.md` and make a deliberate decision
   for each area: public exposure & CORS, auth/authz, request limits & timeouts, database/
   migrations/backups, secrets & environment, observability, API contract, desktop sidecar,
   Docker & release.
3. **Tighten what your exposure model requires.** Common changes: restrict `CorsLayer`
   origins, tune `MAX_BODY_BYTES`/`REQUEST_TIMEOUT`, set a durable `DATABASE_URL`, add a
   reverse-proxy / rate-limit story. Each is security-sensitive (see approval boundaries).
4. **Add tests** for any new validation or authorization path you introduce.
5. **Re-run the audit and `just verify`.** Record what you tightened and what you
   consciously accepted in the generated app's own docs.

## Stop and get human approval before (per AGENTS.md)

Changing CORS, auth/authorization, request-limit, or public-exposure **defaults in the
template** requires explicit maintainer approval — these are template-wide security-posture
changes, not per-app config. In a generated app you own these choices; in the template,
open an issue/PR first. Never commit secrets; inject them through the deployment environment.
