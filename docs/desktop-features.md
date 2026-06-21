# Desktop & cross-platform built-in features — decision record

**Status:** Proposed (research complete; awaiting maintainer ratification per the
"open an issue before changing template defaults" rule in [AGENTS.md](../AGENTS.md)).
**Date:** 2026-06-21.
**Scope:** Which built-in capabilities the App Starter desktop / cross-platform shell
should ship by default, ranked and mapped to the locked v1 scope.
**Method:** A multi-source research pass (web discovery + adversarial verification of
every version-sensitive claim) cross-checked against the repository's resolved
`tauri 2.11.2`. All third-party version/status facts were verified live on 2026-06-21
and will drift — re-verify before acting on the post-v1 items.

---

## 1. The architectural reframe (why this list is unusual)

The desktop app is **not** a classic Tauri app. `desktop/src-tauri/src/main.rs` spawns
the compiled axum server as a **sidecar** (`externalBin`), and the React SPA talks
**HTTP to that embedded backend**. Today the shell ships exactly one plugin
(`tauri-plugin-shell`, for the sidecar), a single window, and `app.security.csp = null`.

Consequence: the highest-value built-ins here are **not** the usual Tauri plugin
checklist (tray, notifications, etc.). They are the **safety and observability defaults
the sidecar-HTTP model specifically needs** — the things that are footguns *because*
there is a bundled HTTP backend. Two principles follow:

1. Any feature must coexist with the sidecar-HTTP-backend model and must not fork or
   dilute the crown-jewel OpenAPI → TypeScript typegen contract.
2. This is a template forked by many projects, so weigh footgun risk and prefer
   safe-by-default for universal wins, scaffold-but-opt-in for product-specific ones.

---

## 2. Decision summary

Lead with desktop **safety + observability defaults** that need no signing and no new
release infrastructure (Tier 1). Treat the entire **signed-distribution + auto-update**
story as one coherent post-v1 epic (Tier 2), gated by the locked v1 deferral of
signing/notarization. Keep product-specific plugins as **pull-when-needed** (Tier 3).
Explicitly **reject** features that fight the architecture or the typegen contract
(Tier 4) — most importantly, **mobile is a hard architectural dead-end** under the
sidecar model, not an effort question.

---

## 3. Tier 1 — Ship now (v1)

High value, low/medium effort, safe-by-default, no signing, no scope conflict.

| Feature | Mechanism | Why it is the right default here |
|---|---|---|
| **Single-instance guard** | `tauri-plugin-single-instance` — register **first** in the Builder; the callback shows/unminimizes/focuses the existing `main` window | The single most important safety default for *this* template. A double-launch otherwise spawns a **second axum sidecar** contending for the same `sqlite://app.db` (`mode=rwc`) and port. Desktop-only — gate with `#[cfg(desktop)]`. |
| **CSP hardening** | Replace `app.security.csp = null` with a strict policy; add a separate `devCsp` for the Vite HMR origin; keep `dangerousDisableAssetCspModification = false` | Closes the one concrete hardening gap. The webview is a full browser pointed at an HTTP backend, so stored-XSS in any field could pivot to IPC or exfiltrate. **`connect-src` must whitelist the sidecar loopback origin** or every fork breaks — ship a known-good policy tested against the Items/Posts examples, not a guess. Tauri does **not** auto-merge a Vite HMR CSP; scaffold `devCsp` explicitly. |
| **Forward the sidecar log stream** | In `main.rs` the sidecar is spawned as `let (_events, child) = …` and `_events` is **discarded**. Drain `CommandEvent::Stdout/Stderr/Terminated` into a log and record the exit code | Highest reliability ROI, unique to this architecture. Today a failed bind, a stale-migration panic, or a non-zero exit shows a silently broken UI with no trace. Draining the stream turns the #1 "desktop app is broken" class into a one-line diagnosis. No new crate. |
| **Window-state persistence** | `tauri-plugin-window-state` — `Builder::default().build()` | Baseline native expectation: reopen where the user left off instead of resetting to 1100x750. Orthogonal to the sidecar/db; writes a small state file. |
| **Frontend error boundary** | `react-error-boundary`, root + route-level, with an offline-safe fallback | Without it a render error white-screens the whole SPA in both browser and desktop webview. Provides the `onError` seam for later opt-in crash reporting. Note: does not catch async/event-handler errors. |
| **Capabilities / ACL tightening** | In `capabilities/default.json` replace the broad `core:default` bundle with an explicit minimal `core:*` allowlist; keep the scoped `shell:allow-execute` sidecar entry; pin `windows:["main"]` and `platforms` | Least-privilege the ACL instead of shipping the broad default bundle. |

**Sequencing.** PR1 (safety + hardening): single-instance + window-state + CSP/`devCsp` +
ACL trim. PR2 (reliability/observability): sidecar-log drain + error boundary, naturally
pulling in `tauri-plugin-log` (the unified sink) and a `/api/health` startup gate as a
fast follow.

---

## 4. Tier 2 — Strong post-v1 (gated by the locked signing deferral or higher risk)

| Feature | Status (verified 2026-06-21) | Notes |
|---|---|---|
| **In-app auto-update** | `tauri-plugin-updater` 2.10.1, official, Tauri-2-native (confirmed) | The "feels finished" feature and #1 Electron-parity expectation. Updates the whole bundle (incl. the sidecar) atomically. **Blocked by v1's signing deferral** — signature verification cannot be disabled. Ship a **default-OFF** scaffold; a placeholder/missing pubkey hard-breaks fork builds. |
| **Updater signing keypair** | minisign, built into `@tauri-apps/cli` 2.11.3 (confirmed) | Mandatory companion. Highest-stakes secret in the desktop story (lose it = can never update installed apps; leak it = attacker pushes malicious updates). Template must **never** commit a key. |
| **`tauri-action` release workflow** | `action-v0.6.2`, 2026-03-14 (confirmed) | **Critical gap:** `release.yml` today builds only bare cargo binaries via rust-embed — **nothing runs `tauri build`**, so the configured `bundle.targets` are aspirational. *Augments* (does not replace) the bare-binary job. Pin exactly: the `latest.json` key format and the Node-24 runner bump changed twice in ~6 months; multi-installer keys need updater ≥ 2.10.0. |
| **In-process axum backend** | Core pattern (no plugin); Tauri-2 compatible (partially-confirmed) | Deletes `bundle-sidecar.sh`, `externalBin`, `tauri-plugin-shell`, and `shell:allow-execute`, removing the duplicate-sidecar/db-lock and port-race classes while preserving the HTTP + typegen contract. Higher-risk spike; loses the "cargo run hot-reload backend / sidecar fails to bind" dev ergonomic. **Not a proven mobile enabler** (see Tier 4). |
| **Local logging baseline** | `tauri-plugin-log` (official, stable, cross-platform) | The natural sink for the sidecar-log drain so shell + backend logs unify. Privacy-safe (on-device). Scaffold with an "enable for crash triage" note; set a sane level + rotation; avoid double-timestamping forwarded sidecar lines. |
| **`/api/health` startup gate** | Reuses the existing readiness probe via the typed client | Turns the inherent startup race (webview up before backend bound) into a deterministic gate. Must treat any healthy responder as success to respect the documented `cargo run` dev fallback. |
| **Tray + native menus + autostart** | Core + `tauri-plugin-autostart` (desktop only; Linux tray needs libayatana-appindicator) | Table-stakes for resident/utility forks, but a footgun for foreground document apps. Tray hide-to-close and menu Quit **must** route through the existing kill-child teardown or you orphan an axum process holding `app.db`. Autostart must default OFF. Scaffold-only. |

---

## 5. Tier 3 — Optional (pull when a fork needs it; not template defaults)

| Feature | Status (2026-06-21) | One-line rationale |
|---|---|---|
| Deep linking | `tauri-plugin-deep-link` 2.4.9 (official, confirmed) | Powerful for OAuth-callback / "open in app", but app-specific. **Only with single-instance shipped first.** |
| Opener helper | `tauri-plugin-opener` | The sanctioned Tauri-2 replacement for the removed shell-open; most defensible of the opener/dialog/fs trio. |
| Preferences store | `tauri-plugin-store` 2.4.x (official) | Legitimate niche the sidecar doesn't cover: UI state that should not round-trip to the API. |
| OS keychain secrets | `keyring` crate 4.x (now via `keyring-core` 1.0.0) **in the sidecar** | Forward-looking secrets path post-Stronghold; matches "data lives in the sidecar." |
| Notifications | `tauri-plugin-notification` (official, incl. mobile) | A CRUD starter has no inherent need; value is product-dependent. |
| Global shortcuts | `tauri-plugin-global-shortcut` 2.3.2 (official) | Inherently dangerous (system-wide clashes); ships zero default permissions by design — wrong for an on-by-default starter. |
| CLI subcommands | `clap` on the existing binary | Nearly free, a named VISION target, reuses the Rust core for ops/automation. |
| PWA packaging | `vite-plugin-pwa` | Cheapest "installable" experience that fully respects the architecture — same SPA, same HTTP contract, zero backend change. |
| Crash reporting | `tauri-plugin-sentry` 0.5.0 + backend `sentry::init` | Community/experimental/pre-1.0; sends telemetry off-device — document **OFF**, opt-in only. Sentry disclaims first-party Tauri support. |

---

## 6. Tier 4 — Avoid (refuted, architecture-incompatible, or footguns for forks)

- **`tauri-plugin-sql` (SQL from the frontend).** Creates a second data path
  (webview → sqlx) that bypasses the OpenAPI contract, typegen, validation, and
  coverage; two writers on one `app.db` cause lock contention; its migration system
  competes with the committed append-only migrations. Actively undermines the crown
  jewel. The sidecar already owns the data layer correctly.
- **`tauri-plugin-stronghold`.** Maintainer-flagged as no longer recommended and
  slated for **removal in Tauri v3** (discussion #7846). Wrong shape (a password-gated
  vault for one or two tokens) and the Tauri-process plugin can't even reach the
  separate sidecar process. Point forks to OS keyring instead.
- **Mobile targets (iOS/Android) + sidecar.** A **hard architectural dead end**, not an
  effort question: iOS forbids spawning child processes and Android throws `os error 2`
  for bundled-binary exec (tauri#9774 — **open**; maintainer "Not yet," 2024-11-28). A
  fork that runs `tauri ios build` gets a UI where every API call fails. The
  prerequisite is the in-process-axum refactor — and even then OS WebView cleartext/ATS
  policy blocks `http://localhost` (Android blocks it by default; iOS needs ATS
  exceptions), so in-process-axum alone does **not** unlock mobile.
- **Local-first sync engines** (cr-sqlite — dormant, no release since Jan 2024; Turso
  `turso::sync` — beta, "data loss possible"; ElectricSQL / PowerSync — Postgres-only,
  non-Rust clients). All require ripping out or bypassing sqlx and a deep data-model
  commitment the "just SQLite via sqlx" design deliberately avoids.
- **On-by-default telemetry/analytics** (Aptabase/PostHog). Consent/GDPR/CCPA liability
  plus accidental phone-home to the template author's key. The clean pattern for this
  architecture is posting events to the fork's own sidecar; document as explicit opt-in,
  ship nothing on by default.
- **Rust-native `#[tauri::command]` as the canonical backend.** Forks the type pipeline
  (OpenAPI for web/Docker vs a separate command-type story) and breaks "the same SPA
  talks HTTP everywhere." Listed only as a last-resort fallback if localhost binding
  ever proves unworkable.
- **IPC Isolation Pattern.** The IPC surface is essentially empty (no
  `#[tauri::command]` handlers); marginal benefit vs real cost. Document as an escape
  hatch only.

---

## 7. Cross-cutting gaps surfaced during review

The primary v1 surface is **Web + Docker**, so do not over-index on the desktop webview.
These belong in the same roadmap:

1. **HTTP security headers on the axum static-file handler** (CSP,
   `X-Content-Type-Options`, frame-ancestors, `Referrer-Policy`, HSTS). The desktop CSP
   only hardens the webview; the larger attack surface is the served SPA. *Most
   defensible omission.*
2. **`SIGTERM`/`SIGINT` graceful shutdown** for clean Docker container stop and
   connection draining.
3. **Sidecar liveness / restart supervision.** Draining `_events` *logs* a crash but
   nothing *restarts* it — the user still sees a broken UI. Diagnosability is not
   recovery.
4. **Dependency audit for the desktop crate.** `desktop/src-tauri` has its **own
   `Cargo.lock`** — an easy `cargo-deny` / Dependabot blind spot.

---

## 8. Evidence boundaries

- **Sourced facts (verified live 2026-06-21):** all plugin versions/status above; the
  mobile-sidecar block (#9774 open; maintainer quote); Stronghold's v3 removal; the
  `release.yml`-builds-no-bundles gap; cr-sqlite dormancy. Verifier corrections folded
  in: cargo-dist is axodotdev/@mistydemeo (not "Astral's fork") and **does not build
  Tauri bundles**; the Sentry plugin is `tauri-plugin-sentry` 0.5.0.
- **Inference (not independently version-checked — pin during implementation):** the
  exact `tauri-plugin-single-instance` version and its `set_focus`-on-hidden workaround
  (tauri#12936), `tauri-plugin-window-state` currency/clamp behavior, the
  `react-error-boundary` React-19 major, the precise CSP directive set, and which
  `core:*` permissions the ACL trim can safely drop. The architectural call on each is
  sound; the version/policy specifics should be confirmed when the PR lands.
- **Freshness:** every external fact is dated 2026-06-21. Re-verify the Tier 2 items
  (updater/tauri-action key formats, mobile-sidecar status) before building — they drift.
