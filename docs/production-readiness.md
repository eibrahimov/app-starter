# Production readiness

App Starter is production-minded, not production-complete. It gives real apps a
small, explicit foundation and leaves high-stakes product and infrastructure
choices visible instead of hiding them behind defaults that may not fit your
domain.

Before exposing a generated app to real users, make deliberate decisions in the areas below.

## Public exposure and CORS

The template ships with `CorsLayer::permissive()` in `src/api.rs` so the
optional Tauri desktop shell can call the sidecar API from `tauri://localhost`.

Before exposing the API publicly:

- decide whether the API is only served through the embedded UI, a desktop sidecar, external clients, or all three;
- restrict allowed origins for public HTTP clients;
- keep the desktop sidecar case in mind before removing the Tauri-aware behavior in `interface/src/api/client.ts`;
- document the chosen exposure model in the generated app.

Changing CORS or public exposure defaults in the template requires explicit maintainer approval.

## Authentication and authorization

The template intentionally ships without auth. Do not treat the example resources as protected.

For a generated app, decide:

- whether users authenticate with sessions, bearer tokens, SSO, local accounts, or another product-specific mechanism;
- where identity enters `AppState` or request extensions;
- which handlers require authentication;
- which operations require authorization beyond identity;
- how tests prove unauthenticated, unauthorized, and authorized cases.

Auth defaults are security-sensitive and should not be added to the template without a proposal and review.

## Request limits and timeouts

The template ships a default request body-size limit and per-request timeout
(`MAX_BODY_BYTES`, `REQUEST_TIMEOUT` in `src/api.rs`). Tune those, and decide
the rest for your workload before public traffic:

- request body size and per-request timeout (shipped — adjust the constants);
- upload/download behavior;
- reverse-proxy timeouts;
- slow-client behavior;
- rate limiting or abuse controls when endpoints are public.

Add tests for validation paths that reject malformed or oversized input.

## Database, migrations, and backups

SQLite is the default operational path. Treat it seriously:

- set `DATABASE_URL` to a durable location outside ephemeral containers;
- mount `/data` or an equivalent volume in Docker deployments;
- run migrations on startup only if that matches your deployment model;
- define backup and restore procedures before storing important data;
- test restore on a throwaway environment;
- never edit committed migrations after they have run anywhere important.

If a generated app needs a different database, prove the need there first. Do
not add database abstraction to the template speculatively.

## Secrets and environment

Use `.env` for local development only. Do not commit secrets.

For deployments:

- inject secrets through the host, container platform, or desktop packaging system;
- keep `.env.example` complete but non-sensitive;
- document required environment variables in the generated app;
- avoid logging secrets or full connection strings.

## Observability and operations

The template ships request tracing, an `x-request-id` layer, graceful shutdown on
SIGTERM, and a database-checking readiness probe at `/api/health`. A production app
should still decide:

- structured log format and retention;
- propagating the request id into your log aggregator or tracing backend;
- error reporting destination;
- liveness vs readiness expectations beyond the bundled `/api/health`;
- database and disk-space alerts for SQLite deployments;
- what information is safe to expose from `/api/health`.

## Frontend and API contract

The backend OpenAPI spec is the source of truth.

For every API change:

- update `#[utoipa::path]` annotations and schemas;
- run `just typegen`;
- commit `interface/src/api/schema.d.ts`;
- run `just check-typegen`;
- update frontend code through the typed `api` client, not raw `fetch`.

## Desktop sidecar

The optional Tauri shell runs the same UI with the server binary as a sidecar.

Before shipping desktop builds:

- change the bundle identifier from `com.example.*`;
- replace placeholder icons;
- decide how the sidecar port is selected and protected;
- verify lifecycle behavior on every supported platform;
- plan codesigning, notarization, and auto-update outside the template defaults.

Do not remove Tauri-aware base URL logic from `interface/src/api/client.ts`
unless the generated app has intentionally dropped desktop support.

## Docker and release

The Dockerfile builds the frontend, embeds it in the Rust binary, and stores SQLite data under `/data`.

Before using release images for real users:

- verify `Cargo.toml` version and release tag alignment;
- run the relevant local gates (`just lint`, `just test`, `just check-typegen`, and `just build`);
- smoke-test the image with a persistent volume;
- confirm image tags and registry ownership;
- decide whether `latest` publishing is acceptable for your release process.

## Pre-public checklist

- [ ] Auth and authorization model chosen and tested.
- [ ] CORS/public exposure model tightened or explicitly accepted.
- [ ] Request limits, timeouts, and validation paths reviewed.
- [ ] SQLite data path, backup, and restore tested.
- [ ] Secrets injected through deployment environment, not committed files.
- [ ] Logging/health/error reporting reviewed for sensitive data.
- [ ] API typegen is current and committed.
- [ ] Docker or desktop release path smoke-tested if used.
- [ ] Generated app documents any intentional deviations from template defaults.
