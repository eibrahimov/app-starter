# Security Policy

## Reporting a vulnerability

Use GitHub's private vulnerability reporting ("Security" tab, "Report a
vulnerability") on this repository. Do not open public issues for security
problems. You will get an acknowledgement within a few days.

## Supported versions

The template tracks its default branch. Generated projects own their own
security posture after `scripts/setup.sh` runs; re-apply template fixes by
cherry-picking.

## Shipped defaults

The template ships some production-minded defaults out of the box: a request
body-size limit and per-request timeout (`MAX_BODY_BYTES`, `REQUEST_TIMEOUT` in
`src/api/mod.rs`), an `x-request-id` layer, graceful shutdown on SIGTERM, a
readiness probe at `/api/health` that checks the database, and a non-root Docker
runtime user.

## Hardening checklist for generated apps

Before exposing an instance publicly:

- Tighten `CorsLayer` in `src/api/mod.rs`. It ships permissive so the Tauri
  desktop shell can reach the sidecar; a public API without the embedded UI
  should restrict origins.
- Add authentication. The template intentionally ships none.
- Tune the body-size limit and request timeout (`MAX_BODY_BYTES`,
  `REQUEST_TIMEOUT` in `src/api/mod.rs`) for your workload, and add rate limiting
  at your reverse proxy.
- Keep dependencies current (Dependabot is preconfigured) and never commit
  `.env`.
