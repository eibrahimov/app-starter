# Security Policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting ("Security" tab, "Report a
vulnerability") on this repository. Do not open public issues for security
problems. You will get an acknowledgement within a few days.

## Supported versions

The template tracks its default branch. After `scripts/setup.sh`, generated
projects own their security posture. Re-apply template fixes by cherry-picking.
See `UPGRADING.md` for the generated-project upgrade workflow.

## Hardening checklist for generated apps

Before exposing an instance publicly:

- Tighten `CorsLayer` in `src/api/mod.rs`. It ships permissive so the Tauri
  desktop shell can reach the sidecar; a public API without the embedded UI
  should restrict origins.
- Add authentication and authorization. The template intentionally ships none.
- Review request body-size limits and timeouts for your workload.
- Define a SQLite backup/restore plan before storing important data.
- Keep dependencies current (Dependabot is preconfigured) and never commit
  `.env`.

See `docs/production-readiness.md` for the broader production hardening map.
Security defaults in the template require discussion before PRs; use the change
request issue template when proposing them.
