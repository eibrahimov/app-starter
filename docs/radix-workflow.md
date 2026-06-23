# Radix build workflow

The end-to-end playbook for turning a natural-language app idea into a working,
gated app on this template's Radix Themes UI. It is the orchestration layer that
ties the rest of the Radix docs, the skills, and the deterministic gates into one
lifecycle an AI agent (or a human) can follow start to finish.

This doc routes; it does not restate. Each stage points at the authoritative
asset for the detail:

- Vocabulary (the four products, every `<Theme>` prop, the component catalog, the
  status-color mapping): [radix-reference.md](radix-reference.md).
- Machine-readable companion (the same vocabulary as JSON, for non-Claude LLM
  tooling): [`interface/src/theme/radix.catalog.json`](../interface/src/theme/radix.catalog.json).
- Day-to-day component how-to: [components.md](components.md).
- Resource recipe (data → API → UI): [authoring-a-plugin.md](authoring-a-plugin.md) and
  the [`add-plugin`](../.claude/skills/add-plugin/SKILL.md) skill.
- How it got here (historical migration): [radix-integration-plan.md](archive/radix-integration-plan.md).

## The lifecycle at a glance

Five stages. Each answers one question, has a primary asset, and exits at a
deterministic gate so the work is resumable by `/loop` and decomposable by
`/workflows`.

| Stage | Question it answers | Primary asset | Exit gate |
| --- | --- | --- | --- |
| 1. Research | Which Radix surface does this touch? | [radix-reference.md](radix-reference.md) (four products) | — (reading) |
| 2. Compare | Themes built-in, primitive gap, or a gated dependency? | the component catalog + the gaps list | — (decision) |
| 3. Plan | What typed, gated units does the request decompose into? | this doc + the skills | — (a checklist) |
| 4. Integrate | Wire each unit through the layers | the matching skill | each skill's bundled `validate-*.sh` |
| 5. Build | Prove it | the gates | `just verify` (+ `just a11y`) |

The golden rule from [AGENTS.md](../AGENTS.md): every unit of work changes one
well-scoped thing and ends green. That is what makes a `/loop` converge and a
restart resume.

## Stage 1 — Research

The first decision in every UI task is **which of the four Radix products** the
request belongs to, because they integrate very differently:

| Product | When the request is about… | Action |
| --- | --- | --- |
| **Themes** (`@radix-ui/themes`) | a button, card, dialog, table, layout, typography | use the component + its props (Stage 4 → `add-component`) |
| **Primitives** (`radix-ui`) | a Toast, Accordion, NavigationMenu, Toolbar, ToggleGroup, Collapsible, Form | compose the unstyled primitive with Theme tokens (a documented gap) |
| **Colors** (`@radix-ui/colors`) | a specific brand hex no built-in hue matches | the custom-accent escape hatch (Stage 4 → `configure-theme`, Path B) |
| **Icons** (`@radix-ui/react-icons`) | a glyph | the allow-list `interface/src/theme/icons.ts` |

Read the "four products" table in [radix-reference.md](radix-reference.md) to
confirm the routing before writing anything.

## Stage 2 — Compare

Compare the request against the **closed catalog** before building. The catalog
is closed on purpose: a finite list is what lets an agent map a need to an exact
component or prop deterministically instead of inventing one. Walk this tree:

```text
Is it a global look-and-feel change (accent, gray, radius, density, panels)?
  └─ yes → configure-theme skill (one edit to theme.config.ts). DONE.
Does a Radix Themes component cover it?  (check the component catalog)
  └─ yes → use it; style via variant/size/color/radius props (add-component).
Is it one of the documented Themes gaps? (Toast/Accordion/NavMenu/Toolbar/…)
  └─ yes → compose a radix-ui primitive + Theme tokens (add-component, gap path).
Does it need a new dependency or an icon-set swap?
  └─ STOP. Approval-gated by AGENTS.md. Do not install; ask first.
```

The component catalog, the gaps list, and the status-color mapping all live in
[radix-reference.md](radix-reference.md); the same data is enumerable as JSON in
[`radix.catalog.json`](../interface/src/theme/radix.catalog.json) for agents that
prefer a structured lookup over prose.

## Stage 3 — Plan

Decompose the natural-language request into **typed, gated units**. A whole-app
prompt decomposes along three axes; map each noun/phrase to a unit and a skill:

| NL signal in the prompt | Unit | Skill |
| --- | --- | --- |
| domain nouns ("projects", "tasks", "invoices") | a REST resource, wired end to end | [`add-plugin`](../.claude/skills/add-plugin/SKILL.md) |
| "add a field / index / status column" to an existing table | a migration | [`add-migration`](../.claude/skills/add-migration/SKILL.md) |
| screens, lists, forms, dialogs, filters | pages + sections + hooks | [`add-component`](../.claude/skills/add-component/SKILL.md) |
| look and feel ("teal", "denser", "rounded", brand hex) | a theme config edit | [`configure-theme`](../.claude/skills/configure-theme/SKILL.md) |

Output of this stage is an **ordered checklist** where each item names its skill
and its exit gate. Order data before UI (a page needs its resource's generated
types). Put the theme edit first or last — it is independent. Example shape:

```text
1. resource: projects        → add-plugin    (just verify + check-typegen)
2. resource: tasks           → add-plugin    (just verify + check-typegen)
3. theme: teal, denser       → configure-theme (lint + build)
4. screen: tasks board       → add-component   (validate-component.sh)
5. a11y sweep                → just a11y
```

Record the checklist somewhere durable (a scratch `.md`, a task list, or the
`/loop` prompt itself) so a restart resumes mid-build.

## Stage 4 — Integrate

Run each unit through its skill. Each skill owns its procedure, invariants, and
footguns and ends with a bundled validation script — this doc only routes you to
the right one and links the detail:

| Unit | Skill | What it owns |
| --- | --- | --- |
| New data resource, end to end | [`add-plugin`](../.claude/skills/add-plugin/SKILL.md) | migration → API → typegen → page + route (and the OpenAPI-registration footgun) |
| Schema change to an existing table | [`add-migration`](../.claude/skills/add-migration/SKILL.md) | append-only migration (never edit a committed one) |
| UI primitive / section / data hook | [`add-component`](../.claude/skills/add-component/SKILL.md) | Themes-first components, props not classes, gap composition |
| Global restyle / brand color | [`configure-theme`](../.claude/skills/configure-theme/SKILL.md) | one `theme.config.ts` edit, or `accent.css` for a brand hex |

Do not cross an [AGENTS.md](../AGENTS.md) approval boundary on the way (see
Guardrails below).

## Stage 5 — Build

Prove the result with the same deterministic gates that gate every change. They
are the contract that makes any AI-produced edit checkable:

```sh
just lint           # cargo fmt --check + clippy -D warnings + Biome + tsc
just test           # backend black-box tests
just check-typegen  # fail if committed TS types are stale (CI enforces)
just verify         # everything CI runs (the blocking gate)
just a11y           # opt-in Playwright + axe page smoke (report-don't-block)
```

`just verify` is the blocking gate; `just a11y` is the opt-in page smoke (it needs
a one-time `bunx playwright install chromium` and runs non-blocking in CI, matching
the repo's report-don't-block stance). Loop Stage 4 → Stage 5 until green; do not
hand off red.

## NL → app: a worked routing example

> **Prompt:** "Build a task tracker — projects each contain tasks with a
> done/not-done state; teal accent and a denser UI."

How an agent routes it through the lifecycle:

1. **Research.** "projects"/"tasks" are data (Themes is not involved yet);
   "teal"/"denser" are global theme. Two surfaces: resources + `<Theme>` config.
2. **Compare.** A task board is `Card`/`Flex`/`Checkbox`/`Button` — all Themes
   built-ins, no gap, no new dependency. "Teal" is the built-in `teal` hue;
   "denser" is `scaling="90%"`. No approval gate triggered.
3. **Plan.** Checklist: resource `projects` → resource `tasks` → theme edit
   (`accentColor: "teal"`, `scaling: "90%"`) → tasks board screen → a11y sweep.
4. **Integrate.** `add-plugin` twice (data + generated types first), then
   `configure-theme` for the two-prop edit, then `add-component` for the board
   page composed from Themes primitives + the typed hooks.
5. **Build.** `just verify` after each resource; `just a11y` after the screen
   lands. Green → done.

Every step is a small, documented, verifiable edit — that is the whole point of
the Radix Themes layer.

## The agent-facing asset map

Everything an agent needs is enumerable and cross-linked. "All assets integrated"
means this table is complete and each row is reachable from the others:

| Asset | Answers | Format |
| --- | --- | --- |
| [radix-reference.md](radix-reference.md) | the full Radix vocabulary | prose + tables |
| [`radix.catalog.json`](../interface/src/theme/radix.catalog.json) | the same vocabulary, parseable | JSON |
| [components.md](components.md) | day-to-day component how-to | prose |
| this doc | the build lifecycle / routing | prose |
| [`add-plugin`](../.claude/skills/add-plugin/SKILL.md) / [`add-migration`](../.claude/skills/add-migration/SKILL.md) / [`add-component`](../.claude/skills/add-component/SKILL.md) / [`configure-theme`](../.claude/skills/configure-theme/SKILL.md) | the step-by-step procedures | skills |
| `interface/src/theme/theme.config.ts` | the global theme config surface | code |
| `interface/src/theme/icons.ts` | the closed icon allow-list | code |
| `interface/src/theme/accent.css` | the custom-brand escape hatch | code (templated-empty) |
| `just verify` / `just a11y` | is it correct? | gates |

## `/loop` and `/workflows` playbooks (building)

Because each Stage-4 unit ends at a gate (the golden rule above), the build is
loop- and workflow-safe. The general mechanics — and the migration-era examples —
live in [radix-integration-plan.md](archive/radix-integration-plan.md); the
build-specific shapes are:

- **`/loop`** (self-paced, one unit per iteration): `/loop add the next unbuilt
  resource from spec.md, then run just verify` — scaffold, gate, repeat until the
  spec is wired. Swap `add-plugin` for `add-component` to loop over screens, or
  `just a11y` to iterate to zero axe violations.
- **`/workflows`** (opt-in, deterministic fan-out): pipeline each resource through
  scaffold → verify (use `isolation: "worktree"` if parallel scaffolds would
  collide on `src/api.rs` / `src/lib.rs` / the router); or build each screen, then
  adversarially audit it for Themes-conformance and contrast.

## Guardrails

The build path must never silently cross an [AGENTS.md](../AGENTS.md) approval
boundary — new dependencies, an icon-set swap, security/CORS/auth/limits changes,
migration-history edits, hand-editing the generated `schema.d.ts`, or removing
anything from the additive-only `/api/v1`. AGENTS.md is the authoritative list;
get human sign-off there first. When in doubt, the gate is the default: do not
proceed past a red `just verify`.
