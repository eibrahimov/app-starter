# App Starter direction and v1 readiness

App Starter is an opinionated foundation for serious Rust + TypeScript
applications without hiding the system behind framework magic. The v1 path
should protect what already makes it valuable: one understandable app, one
binary, one API contract, and a clear way to grow from examples into a real
domain.

## Decision standard

For any proposed change, ask:

> Is this good for App Starter, or bad for App Starter?

A change is good when it makes generated projects easier to start, safer to
ship, clearer to maintain, and easier for humans and AI agents to improve. A
change is bad when it adds surface area while increasing confusion, lock-in,
maintenance cost, security ambiguity, or template bloat.

## Product identity to protect

- **One Rust server owns the backend and serves the embedded UI.** The default
  deployment story stays simple: build one binary, optionally containerize it,
  and run it with SQLite.
- **The backend is the source of truth for API types.** OpenAPI annotations and `just typegen` keep frontend calls honest.
- **Examples teach the architecture.** `items` and `posts` are worked examples,
  not product features. Generated apps can copy, replace, and eventually delete
  them.
- **SQLite-first is a feature, not a limitation.** It keeps local development,
  Docker deployment, and small-team operations understandable. Multi-database
  abstractions need strong evidence before entering the template.
- **Optional platform surfaces stay optional.** Docker and Tauri are delivery paths, not reasons to complicate the core web/API foundation.
- **Explicit code beats hidden framework behavior.** Generated apps should be easy to inspect six months later.

## What is already strong

- Clear full-stack spine: axum API, sqlx migrations, embedded React UI, OpenAPI-generated TypeScript.
- Two end-to-end resource examples that cover minimal CRUD and a richer lifecycle/list/stats flow.
- Useful validation commands and CI for Rust formatting, clippy, backend tests,
  frontend typecheck/build, and typegen drift.
- Docker and Tauri paths exist without forcing every app into those paths.
- Project guidance already documents the most common OpenAPI registration failure and migration/typegen rules.

## Before v1: highest-value improvements

These are the changes most likely to improve adoption and shipping readiness within four weeks.

1. **First-run confidence**
   - Make setup expectations explicit in `README.md`.
   - Explain that `scripts/setup.sh` is a one-time fresh-template initializer.
   - Add screenshots or a short demo once the UI is stable enough to represent the template well.

2. **Human resource-building path**
   - Keep `docs/add-a-resource.md` aligned with `AGENTS.md`.
   - Make the end-to-end recipe understandable without reading agent-only guidance.
   - Include common mistakes and verification checkpoints.

3. **Production boundary map**
   - Keep `docs/production-readiness.md` honest: production-minded, not production-complete.
   - Document seams for auth, CORS, request limits, database backup/restore,
     secrets, observability, Docker, and desktop sidecar behavior.

4. **Generated-project upgrade story**
   - Maintain `UPGRADING.md` so generated apps can pull fixes without rerunning setup or blindly copying template files.
   - Release notes should call out generated-project impact when relevant.

5. **Contribution loop from real apps**
   - Use issue templates for adoption friction and generated-app backports.
   - Prefer evidence from real projects over speculative abstractions.
   - Port patterns back to `items`/`posts`; never copy domain-specific app code into the template.

6. **Fresh-clone trials before v1**
   - Ask at least three people or agents who did not build the template to create
     a project, run it, add one resource, run typegen/tests, and identify what
     confused them.
   - Fix repeated friction in docs or defaults before adding new features.

## Defer until repeated evidence appears

These may become important, but adding them too early would likely weaken v1.

- Built-in authentication, users, roles, organizations, tenancy, or billing.
- PostgreSQL/MySQL abstraction layers, repository traits, or generic service frameworks.
- CRUD generators or a template-specific CLI.
- Multiple frontend frameworks or JS package managers.
- Kubernetes, Terraform, or a large cloud-provider matrix.
- A heavy design system or admin shell that users must undo.
- Desktop auto-update/codesigning workflows beyond clear optional guidance.
- Native mobile release automation beyond the current Tauri starting point.

## Changes that require explicit approval

Do not make these as incidental cleanups:

- Security posture, authentication, authorization, CORS, request limits, or public exposure defaults.
- Dependency policy, Rust edition/toolchain, JS package manager, or major framework upgrades.
- Release workflow, Docker publishing behavior, tags, binary/package names, or registry targets.
- Migration history, data-destructive defaults, or committed migration edits/renames.
- Generated-file policy or manual edits to `interface/src/api/schema.d.ts`.
- Architectural conventions not demonstrated by both worked examples.
- Formal governance systems such as CODEOWNERS, branch protection policy, support windows, or required labels.
- License policy or copied third-party code.

## Decision tests for future proposals

Use these before accepting new template scope:

- Does this reduce first-project friction without hiding important behavior?
- Does this help generated apps ship safely, not just demo well?
- Does this preserve the one-binary + OpenAPI-source-of-truth identity?
- Can a user delete it if they do not need it?
- Does it work for both `items` and `posts`, or is it domain-specific?
- Would it still look maintainable six months after setup?
- Does it improve AI-agent safety by making intent, boundaries, and validation clearer?
- What simpler alternative was considered?
- What future scope does this invite, and are we ready to own it?

## AI-native operating model

AI agents should be able to make routine changes safely, but they need explicit
boundaries. The template should continue to encode:

- where to add code for each layer;
- which files are generated or append-only;
- what validation commands prove correctness;
- what requires human approval;
- how to report skipped checks and residual risk;
- how to turn recurring generated-app friction into issues, docs, or template changes.

The goal is not autonomous churn. The goal is better human/agent collaboration
where the repo itself makes correct work easier than incorrect work.
