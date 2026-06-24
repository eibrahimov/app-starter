# Plugin Framework — Research Report

> Companion to [`plugin-framework.md`](plugin-framework.md). Multi-agent web
> research (5 parallel angles, ~55 unique cited sources) validating the design
> against how real-world plugin ecosystems are built. Each claim is cited;
> disagreements between sources are called out rather than smoothed over.
>
> **Re-reviewed** by a 6-agent adversarial panel — see
> [`plugin-framework-review.md`](plugin-framework-review.md). Citation integrity
> was found high (no fabricated/dead URLs); minor corrections from that review are
> marked inline below.
>
> **Date:** 2026-06-22. **Method:** scope → 5 parallel search agents
> (architecture · ecosystems/contracts · DB/migrations · frontend · security/
> governance) → claim extraction → cross-source synthesis.
>
> **Bottom line:** the compile-time core (`inventory` + `utoipa-axum` + Vite
> `import.meta.glob`) is **confirmed** as the right architecture for a
> single-binary, typed-contract app. Research **changed two design decisions**
> (migration ownership; API/OpenAPI namespacing) and **added** several controls
> (host-compat version, honest trust model, supply-chain gates). See §6.

## 1. Verdict: compile-time vs runtime

For a self-contained binary whose value proposition is end-to-end type safety
from one OpenAPI contract, **compile-time plugins are the correct default**, for
three evidence-backed reasons:

- **Rust has no stable ABI** — layout "may break not only between compiler
  versions, but also compiler runs" ([abi_stable]), which is why runtime
  `.so`/`.dll` loading (`libloading`/`abi_stable`) is version-locked, and
  `abi_stable` explicitly "doesn't support async" — a non-starter for an axum
  app. Runtime native plugins are out.
- **The contract stays complete only if every route is known at build time.**
  Compile-time registration keeps the OpenAPI→typegen pipeline coherent; a
  runtime plugin would add endpoints the generated `schema.d.ts` never sees.
- **Single artifact preserved.** This is the textbook *modular monolith* —
  Shopify's deliberate choice: "keep all of the code in one codebase, but ensure
  that boundaries were defined and respected," each component "available through
  a public API" ([Shopify Eng]).

**When runtime *would* be right (the escape hatch):** the moment the goal
becomes **untrusted third-party** plugins or **post-deploy extensibility without
a rebuild**, the answer is *not* dynamic native libraries but the **WASM
Component Model / Extism** — in-process, capability-sandboxed ("a WASM module
starts with zero capabilities and must be explicitly granted each one"
[Extism]), language-agnostic. That is a separate additive boundary, not a
replacement for the typed compile-time core. The research agreed across angles:
"the real superpower of backend WASM is not speed — it's isolation"
([Backend WASM 2025]).

This confirms the proposal's rejection of runtime/WASM loading for the core
(`plugin-framework.md` §8) — with the refinement that WASM is the *right* future
escape hatch if untrusted plugins ever become a goal, which §6 records.

## 2. Patterns to adopt, by area (with precedents)

### 2.1 Registration mechanism — `inventory` (confirmed, with guards)

`inventory` is the right pick over `linkme` here because it "supports submitting
non-const expressions" and supports WASM/`dlopen`, whereas `linkme` is
const-only and "does not currently support WASM" ([life-before-main]). It is
proven in production — `typetag` is built on it ([typetag]). `utoipa-axum`'s
`OpenApiRouter` is **purpose-built** for the per-plugin pattern: it carries both
the axum route *and* its OpenAPI paths/schemas and composes via `.nest()`/
`.merge()`, then splits back into `(Router, OpenApi)` ([utoipa-axum]). This
directly validates lifting the `.nest`/`.merge` ban (`plugin-framework.md` §6).

**Two hard cautions the research surfaced** (now design requirements, §6):

- **`inventory` registers via life-before-main constructors and fails
  *silently*** on unsupported targets — "other platforms will simply find that
  no plugins have been registered" ([inventory]) — and link-section registries
  are **sensitive to dead-code elimination** because items rely on `#[used]`
  ([life-before-main]); `linkme` even has a known cross-crate discard bug
  ([linkme#36]). An empty registry is not a compile error. **Mitigation:** gate
  plugins behind Cargo features *and* add a test asserting the expected plugin
  set is present at runtime — never trust the registry to be non-empty.
- The whole pattern is **contested upstream** (the Rust "global registration"
  pre-RFC exists precisely because both `ctor` and linker-section approaches
  have acknowledged downsides). Acceptable, but document it.

### 2.2 API contract & namespacing — namespace *by construction* (refines design)

The convergent lesson across VS Code, Backstage, Kubernetes, Grafana, Terraform,
and Shopify:

1. **Make compatibility explicit and machine-checked.** Every mature system
   pins plugin↔host compat with one declared, enforced value: VS Code's
   `engines.vscode` semver range, Grafana's `grafanaDependency` ("validated
   using node-semver", enforced at install in v12), Terraform/go-plugin's
   integer **protocol version** that "can be incremented to invalidate any
   previous plugins" with "a human friendly error message." There is **no
   industry consensus on the *unit*** (semver range vs protocol int vs Shopify's
   dated versions) — only that there is *one explicit, checkable* declaration
   and load fails loudly on mismatch. **→ Add a `host_api` compat field to
   `plugin.toml`, checked at startup.**
2. **Namespace by construction, not convention.** Kubernetes makes collisions
   *structurally impossible*: the route is mechanically derived from a
   reverse-DNS group (`/apis/<group>/<version>/<plural>`). WordPress relies on
   social prefix convention and sources call it error-prone. **→ Derive both the
   route prefix *and* the OpenAPI component names from the plugin id.**
3. **The OpenAPI-merge footgun is real and specific:** when combining specs,
   same-named components are **silently last-wins-overwritten**; the fix is
   per-source namespace prefixing so `User` from plugin `guestbook` becomes
   `guestbook_User` with all `$ref`s rewritten ([Speakeasy], [Redocly]). **→
   Plugin schemas must be generated under a per-plugin component prefix**, or two
   plugins each defining `Item`/`Settings` silently corrupt the shared client.
4. **Separate contract from implementation** (Backstage publishes extension
   points in a `-node` package so modules depend on the contract, not the impl)
   and **tier the surface** (VS Code's "proposed API" plugins must opt into and
   *cannot publish against*; only stable is never-break).

### 2.3 Database migrations — **per-plugin tracking table (design changed)**

This is where research **overturned the doc's recommended default**. The doc's
§5 recommended a shared `_sqlx_migrations` table with a collision guard. The
evidence shows running one `Migrator` per plugin against the shared table is the
**documented failure mode**, not a tunable risk:

- sqlx's `validate_applied_migrations` returns `VersionMissing` when an applied
  migration isn't in the current Migrator's set — so each plugin's Migrator sees
  the *other* plugins' rows as "missing" and errors ([sqlx migrator.rs]). This is
  open issue **[sqlx#1698]** verbatim ("previously applied but is missing in the
  resolved migrations") and **[sqlx#3573]** (request for multiple sources, which
  was *closed*).
- The only built-in escape, `ignore_missing(true)`, "silences genuine drift" too
  — a blunt instrument ([sqlx migrator.rs]).
- sqlx's own planned fix ([sqlx#3565]) is to **change the migrations-tracking
  table name per app**; its Postgres "schema per app" form is the interim
  workaround, and **schemas are Postgres-only — SQLite has none.** So for this
  SQLite app the analogue is a **per-plugin tracking table** via
  `Migrator::dangerous_set_table_name("_sqlx_migrations_<plugin>")`.

> **Correction from re-review** (§2.2 B2): that method does **not exist in the
> pinned `sqlx 0.8.6`** — it first ships in **0.9.0**. So this is *not* achievable
> on the repo today; it requires a breaking, approval-gated `sqlx 0.9` upgrade, or
> a 0.8-compatible alternative (ATTACH-per-file database, or a custom per-plugin
> versioned-SQL applier). The diagnosis above is correct; the prescription needs
> this decision plus a startup `busy_timeout`/WAL story and a plugin-table-name
> collision guard.

Cross-framework, the split is clear: **Django** tags each log row with its
`app_label`, namespacing migration history per app (by *tagging* — there is no
unique constraint on `(app_label, name)`, so it's convention-by-tag, not
schema-enforced; the re-review corrected the earlier "impossible by design"
wording); **Rails** engines namespace *tables* (`isolate_namespace` →
`my_engine_articles`) but share the log relying on unique timestamps; **WordPress** skips a log entirely (prefix tables with
`$wpdb->prefix`, store a per-plugin version int in `options`, diff via
`dbDelta`). **Discourse explicitly discourages plugin-owned tables** because of
"what happens when someone uninstalls the plugin?" and steers to a key/value
`PluginStore` ([Discourse]).

**Adopted pattern (now design default):** (a) **per-plugin tracking table**
(`_sqlx_migrations_<plugin>`) so each plugin owns its own version keyspace —
removes the collision entirely; (b) **table-name prefixing** `<plugin>_<table>`
as the primary collision-avoidance for the data tables themselves (Rails/
WordPress); (c) **plugins may FK *to* core tables, core never depends on plugin
tables**; (d) **decide an uninstall policy up front** (orphan vs teardown).

### 2.4 Frontend composition — `import.meta.glob` (confirmed)

Build-time discovery is right and well-supported: glob patterns "must be passed
as literals" and are statically analyzed at build ([Vite]); matches are lazy
code-split by default or eager-bundled with `{ eager: true }`. Decisively, every
plugin shares the host's **single** React/Router/Query/Radix instances *by
construction* — sidestepping the entire failure class that dominates runtime
composition: Module Federation/single-spa require `singleton: true` for React or
hit the "Invalid hook call" multi-instance error, and silent version-skew
breakage without `strictVersion` ([Module Federation], [single-spa]). **Backstage's
own frontend system trends *toward* this** — plugins install as build-time deps
contributing route/page descriptors, and explicit nav extensions were deprecated
in favor of auto-discovery ([Backstage frontend]). The doc's `import.meta.glob`
choice matches the direction the most relevant precedent took.

### 2.5 Security, governance & DX — name the trust model honestly

> **Corrections from re-review** ([`plugin-framework-review.md`](plugin-framework-review.md)
> §2 M6/M7, §5): the cited crates.io incidents below executed at **runtime**, not
> build time, so they evidence the *runtime* in-process risk, not the `build.rs`
> vector (which is dangerous *by design*, cited generically). `rustdecimal` was
> **2022** (RUSTSEC-2022-0042), not 2025. The trust statement also needs the
> explicit in-process reach (DB file / env / network egress) added; and the
> governance guidance below is **marketplace-shaped and does not map to a
> compile-time crate model** — see the review.

- **Trust model (must be stated, not implied away):** a compiled-in crate has
  full process access. It can read the SQLite file directly, read environment/
  secrets, and open its own listeners or outbound connections — entirely outside
  the CORS/body-limit/timeout layers, which govern only HTTP traffic through the
  shared router and provide **no containment** against in-process code. Worse,
  **`build.rs`/proc-macros run arbitrary code at *build* time, before anything is
  loaded** — build scripts can "exfiltrate environment variables (including CI
  secrets)" ([systemshardening]); this is a property of build scripts by design,
  not tied to any one incident. Obsidian (the closest unsandboxed analog) says
  plainly it "cannot reliably restrict plugins" ([Obsidian security]). A
  permission *manifest* in an unsandboxed host is **disclosure, not enforcement** —
  only 39.8% of Chrome extensions met least-privilege *with* a manifest system
  ([Springer]). "Trusted plugins only" is the sole real control; don't market a
  manifest as a sandbox.
- **Supply chain controls are build-graph, since runtime isolation is
  impossible:** run **`cargo-deny`** (license/source/banned-crate, already in
  `just verify`) + **`cargo-audit`** as reactive backstops, and **`cargo-vet`**
  (human audit-before-entry, importable Google/Mozilla audit sets — [cargo-vet])
  over the workspace's own dependency graph. These are reactive (deny/audit) or
  human-gated (vet) — none stop a novel malicious crate or `build.rs` payload.
  crates.io's smaller scale lowers but doesn't remove typosquat risk (e.g.
  `faster_log`/`async_println`, Sept 2025 — both **runtime** payloads; `rustdecimal`,
  2022). The *defining* class is maintainer/CI compromise: event-stream (2018, a
  social-engineering maintainer handover) and TanStack (2026, a CI/Actions OIDC-token
  compromise — "no npm tokens were stolen" — 84 malicious versions).
- **Governance that correlates with healthy ecosystems:** automated scanning on
  *every published version* (VS Code study: ~5.61% of 52,880 extensions
  potentially harmful, 613M installs — "high install count" is worthless as a
  safety signal), manual review reserved for popular/featured/flagged plugins
  (Obsidian, WordPress), and **distributed code ownership with explicit
  namespace rules** — cited as a "key best practice contributing to Backstage's
  success."
- **DX:** `cargo-generate` (Liquid templates + Rhai hooks) is the idiomatic Rust
  scaffolder — the analog of `yo code` — validating the proposal's scaffolding
  skill/CLI. Ship the two worked-example plugins as teaching templates and an
  isolated standalone build/test loop.

## 3. Cross-source disagreements (resolved)

| Tension | Resolution for this project |
|---|---|
| Compat unit: semver range vs protocol int vs dated | Use a **semver range against a declared host plugin-API version** (VS Code/Grafana model); simplest to check in Rust + npm. |
| Collision prevention: structural (k8s) vs convention (WordPress) | **Structural** — derive prefixes from plugin id; never rely on author discipline. |
| Plugins owning DB tables: yes (WP/Django/Rails) vs no (Discourse) | **Yes, but prefixed + per-plugin tracking table + explicit uninstall policy.** Discourse's objection is really "no uninstall story" — so define one. |
| Frontend runtime independence (MF/single-spa) vs build-time | **Build-time** — single-binary has no independent-deploy requirement; avoids singleton/version-skew hell. |
| Manifest as security control vs disclosure | **Disclosure only** — never present as enforcement absent a sandbox. |
| `inventory` vs `linkme` | **`inventory`** (non-const + WASM), with a runtime presence test to cover silent-empty/DCE. |

## 4. Top risks & mitigations

1. **Silent empty registry** (inventory + DCE/LTO/unsupported target) → Cargo
   feature gating + a test asserting expected plugins are registered.
2. **OpenAPI component collision** (two plugins define `Item`) silently corrupts
   the generated client → per-plugin component-name prefix; extend the parity
   guard to assert no cross-plugin schema-name clashes.
3. **sqlx migration `VersionMissing`** across plugins → per-plugin tracking
   table; never multiple Migrators on the shared `_sqlx_migrations`.
4. **Supply-chain via `build.rs`/proc-macro** before load → `cargo-vet` gate;
   treat build scripts as the highest-risk review surface.
5. **Untrusted UI in host realm** (no sandbox) → restrict to trusted/reviewed
   plugins; if untrusted UI is ever needed, isolate in iframe/Worker, not glob.

## 5. Recommended edits to `plugin-framework.md`

Applied in this commit (evidence-backed corrections to a draft):

- **§5 Migrations** — default flipped to **per-plugin tracking table**
  (`_sqlx_migrations_<plugin>`); the shared-table approach demoted with the
  sqlx#1698/#3573 citation explaining *why* it's a failure mode, not a tunable
  risk. Add table-prefix + uninstall-policy requirements.
- **§3 API contract** — add (a) per-plugin **OpenAPI component-name prefixing**
  (the silent last-wins merge footgun), and (b) a **`host_api` compat field** in
  `plugin.toml`, checked at startup with a loud error.

Deferred to implementation (recorded as new open questions/§9 items, not yet
written into the design body — they expand scope and want sign-off):

- A **"stable vs proposed" API tier** for the plugin-facing surface (VS Code).
- A **runtime presence test** + Cargo-feature gating for `inventory` (silent-empty).
- **`cargo-vet`** as the blessed-plugin gate; an honest **trust-model
  disclosure** section; a plugin **capability manifest** (disclosure only).

## 6. Net assessment

The design's spine is sound and matches how production systems are actually
built — compile-time registration, `utoipa-axum`, build-time UI discovery, and
crate-based plugins are each independently validated by a real precedent. The
research's value was at the edges: it caught one recommendation that was
*documented-broken* (shared migration table), one *silent-corruption* gap
(OpenAPI component names), one *silent-failure* gap (empty inventory registry),
and it sharpened the honest framing of the trust model. None of these change the
architecture; all of them are now folded into the plan.

---

## Sources

**Architecture/registration:** [inventory](https://github.com/dtolnay/inventory) ·
[linkme](https://github.com/dtolnay/linkme) · [linkme#36](https://github.com/dtolnay/linkme/issues/36) ·
[life-before-main](https://grack.com/blog/2026/06/11/life-before-main/) ·
[typetag](https://github.com/dtolnay/typetag) ·
[Bevy Plugin](https://docs.rs/bevy_app/latest/bevy_app/trait.Plugin.html) ·
[abi_stable](https://crates.io/crates/abi_stable) ·
[utoipa-axum](https://docs.rs/utoipa-axum/latest/utoipa_axum/router/struct.OpenApiRouter.html) ·
[Extism FAQ](https://extism.org/docs/questions/) ·
[go-plugin](https://github.com/hashicorp/go-plugin) ·
[Backend WASM 2025](https://thebackenddevelopers.substack.com/p/wasm-on-the-backend-in-2025-sandboxing) ·
[Shopify Eng](https://shopify.engineering/deconstructing-monolith-designing-software-maximizes-developer-productivity)

**Ecosystems/contracts:** [VS Code proposed API](https://code.visualstudio.com/api/advanced-topics/using-proposed-api) ·
[VS Code API process](https://github.com/microsoft/vscode/wiki/Extension-API-process) ·
[VS Code contribution points](https://code.visualstudio.com/api/references/contribution-points) ·
[Backstage extension points](https://backstage.io/docs/backend-system/architecture/extension-points/) ·
[WordPress hooks](https://learn.wordpress.org/lesson/developing-with-hooks/) ·
[Kubernetes CRD versioning](https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definition-versioning/) ·
[Terraform plugin protocol](https://developer.hashicorp.com/terraform/plugin/terraform-plugin-protocol) ·
[Grafana plugin.json](https://grafana.com/developers/plugin-tools/reference/plugin-json) ·
[Grafana version enforcement](https://grafana.com/whats-new/2025-05-05-enforcing-stricter-version-compatibility-checks-in-plugin-cli-install-commands/) ·
[Shopify versioning](https://shopify.dev/docs/api/usage/versioning) ·
[Speakeasy merge](https://www.speakeasy.com/docs/sdks/prep-openapi/merge) ·
[Redocly combine](https://redocly.com/blog/combining-openapis)

**DB/migrations:** [sqlx migrate! macro](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html) ·
[sqlx migrator.rs](https://raw.githubusercontent.com/launchbadge/sqlx/main/sqlx-core/src/migrate/migrator.rs) ·
[sqlx#1698](https://github.com/launchbadge/sqlx/issues/1698) ·
[sqlx#3573](https://github.com/launchbadge/sqlx/issues/3573) ·
[sqlx#3565](https://github.com/launchbadge/sqlx/discussions/3565) ·
[Django migrations](https://docs.djangoproject.com/en/6.0/topics/migrations/) ·
[Rails::Engine](https://api.rubyonrails.org/classes/Rails/Engine.html) ·
[WordPress creating tables](https://developer.wordpress.org/plugins/creating-tables-with-plugins/) ·
[Discourse plugin migrations](https://meta.discourse.org/t/plugin-database-migrations/259521) ·
[Bytebase multi-tenant](https://www.bytebase.com/blog/multi-tenant-database-architecture-patterns-explained/)

**Frontend:** [Vite glob](https://vite.dev/guide/features) ·
[Module Federation shared](https://module-federation.io/configure/shared) ·
[MF runtime troubleshooting](https://module-federation.io/guide/troubleshooting/runtime) ·
[single-spa setup](https://single-spa.js.org/docs/recommended-setup/) ·
[Backstage frontend plugins](https://backstage.io/docs/frontend-system/architecture/plugins/) ·
[Backstage routes](https://backstage.io/docs/frontend-system/architecture/routes/) ·
[Figma how plugins run](https://developers.figma.com/docs/plugins/how-plugins-run/) ·
[Figma plugin security](https://www.figma.com/blog/an-update-on-plugin-security/) ·
[VS Code webview](https://code.visualstudio.com/api/extension-guides/webview) ·
[Trail of Bits VSCode](https://blog.trailofbits.com/2023/02/21/vscode-extension-escape-vulnerability/)

**Security/governance/DX:** [cargo-vet](https://mozilla.github.io/cargo-vet/) ·
[Google crate audits](https://opensource.googleblog.com/2023/05/open-sourcing-our-rust-crate-audits.html) ·
[Rust supply-chain](https://www.systemshardening.com/articles/cicd/rust-cargo-supply-chain-security/) ·
[RustSec](https://rustsec.org/) ·
[crates.io malicious crates](https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/) ·
[event-stream analysis](https://www.rescana.com/post/in-depth-analysis-supply-chain-poisoning-of-popular-npm-packages-exploiting-event-stream-ua-parser/) ·
[TanStack postmortem](https://tanstack.com/blog/npm-supply-chain-compromise-postmortem) ·
[Chrome least-privilege study](https://link.springer.com/article/10.1007/s10207-022-00610-w) ·
[Obsidian security](https://help.obsidian.md/plugin-security) ·
[Obsidian future of plugins](https://obsidian.md/blog/future-of-plugins/) ·
[VS Code extension study](https://arxiv.org/html/2411.07479v1) ·
[Backstage ecosystem health](https://tldrecap.tech/posts/2026/backstagecon-europe/backstage-plugin-ecosystem-sustainability/) ·
[cargo-generate](https://github.com/cargo-generate/cargo-generate)

[abi_stable]: https://crates.io/crates/abi_stable
[Extism]: https://extism.org/docs/questions/
[Backend WASM 2025]: https://thebackenddevelopers.substack.com/p/wasm-on-the-backend-in-2025-sandboxing
[Shopify Eng]: https://shopify.engineering/deconstructing-monolith-designing-software-maximizes-developer-productivity
[life-before-main]: https://grack.com/blog/2026/06/11/life-before-main/
[typetag]: https://github.com/dtolnay/typetag
[inventory]: https://github.com/dtolnay/inventory
[linkme#36]: https://github.com/dtolnay/linkme/issues/36
[utoipa-axum]: https://docs.rs/utoipa-axum/latest/utoipa_axum/router/struct.OpenApiRouter.html
[Speakeasy]: https://www.speakeasy.com/docs/sdks/prep-openapi/merge
[Redocly]: https://redocly.com/blog/combining-openapis
[sqlx migrator.rs]: https://raw.githubusercontent.com/launchbadge/sqlx/main/sqlx-core/src/migrate/migrator.rs
[sqlx#1698]: https://github.com/launchbadge/sqlx/issues/1698
[sqlx#3573]: https://github.com/launchbadge/sqlx/issues/3573
[sqlx#3565]: https://github.com/launchbadge/sqlx/discussions/3565
[Discourse]: https://meta.discourse.org/t/plugin-database-migrations/259521
[Vite]: https://vite.dev/guide/features
[Module Federation]: https://module-federation.io/configure/shared
[single-spa]: https://single-spa.js.org/docs/recommended-setup/
[Backstage frontend]: https://backstage.io/docs/frontend-system/architecture/plugins/
[systemshardening]: https://www.systemshardening.com/articles/cicd/rust-cargo-supply-chain-security/
[Obsidian security]: https://help.obsidian.md/plugin-security
[Springer]: https://link.springer.com/article/10.1007/s10207-022-00610-w
[cargo-vet]: https://mozilla.github.io/cargo-vet/
