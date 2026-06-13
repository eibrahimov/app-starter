# Security Policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting ("Security" tab, "Report a
vulnerability") on this repository. Do not open public issues for security
problems. You will get an acknowledgement within a few days.

## Supported versions

The template tracks its default branch. Generated projects own their own
security posture after `scripts/setup.sh` runs; re-apply template fixes by
cherry-picking.

## Hardening checklist for generated apps

Before exposing an instance publicly:

- Tighten `CorsLayer` in `src/api/mod.rs`. It ships permissive so the Tauri
  desktop shell can reach the sidecar; a public API without the embedded UI
  should restrict origins.
- Add authentication. The template intentionally ships none.
- Review request body-size limits and timeouts for your workload.
- Keep dependencies current (Dependabot is preconfigured) and never commit
  `.env`.
