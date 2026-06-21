---
name: cut-release
description: >-
  Cut a versioned App Starter release: bump the version, run the gates, commit, and
  tag vX.Y.Z so CI publishes the multi-arch GHCR image and attaches Linux/macOS/
  Windows binaries to the GitHub Release. Use when asked to cut or ship a release, bump
  the version, tag a version, or publish a new version. Runs a read-only preflight that
  catches the Cargo.toml/tag mismatch the release workflow hard-fails on.
---

# Cut a release

Operational procedure for tagging a release. The CI behavior is defined in
[`.github/workflows/release.yml`](../../../.github/workflows/release.yml); deployment
context is in [`README.md`](../../../README.md) (Deploy) and
[`docs/production-readiness.md`](../../../docs/production-readiness.md) (Docker and release).

## How releases work here

Pushing a `v*` tag triggers `release.yml`, which **verifies the tag matches the
`Cargo.toml` version** (`v1.2.3` requires `version = "1.2.3"`), then builds and pushes a
multi-arch image to GHCR and attaches prebuilt binaries to the GitHub Release. A
tag/version mismatch fails the workflow — the preflight below catches it locally first.

## Procedure

1. **Decide the bump** (semver): patch for fixes, minor for additive features/endpoints,
   major only for a breaking change — which for the API means opening `/api/v2` (per
   AGENTS.md; get approval first).
2. **Bump the version in lockstep** across every manifest that carries it. The release
   CI only publishes the backend, but a stale value elsewhere ships a wrong-versioned
   desktop bundle — the preflight in step 4 fails if these disagree:
   - `Cargo.toml` (`version = "X.Y.Z"`), then refresh the lockfile:
     ```sh
     cargo update -p app-starter
     ```
   - `interface/package.json` and `desktop/package.json`
   - `desktop/src-tauri/Cargo.toml` (the `app-starter-desktop` crate), then:
     ```sh
     cargo update -p app-starter-desktop
     ```
   - `desktop/src-tauri/tauri.conf.json` (top-level `version`; the dmg/installer filename
     is derived from it)
   - `interface/e2e/a11y.spec.ts` (the `HEALTH` mock's `version`; cosmetic — keep it truthful)
3. **Run the full gate set:**
   ```sh
   just verify
   ```
4. **Preflight** (read-only) before tagging — confirms a clean tree, Cargo.toml/Cargo.lock
   agreement, that the interface/desktop manifests all match the bumped version, and that
   the tag does not already exist:
   ```sh
   .claude/skills/cut-release/scripts/preflight-release.sh
   ```
5. **Commit** with a conventional message: `chore(release): bump version to X.Y.Z`.
6. **Tag and push** (this triggers publishing — an outward-facing action; confirm intent):
   ```sh
   git tag vX.Y.Z
   git push origin master --tags
   ```
7. **Verify** the Release workflow goes green and the GHCR image + binaries appear.

## Stop and get human approval before (per AGENTS.md)

Release workflow changes, Docker publishing, tags, package/binary names, and registry
targets are approval-gated. Tagging publishes to a public registry — treat the push as an
outward-facing action and confirm before running it.
