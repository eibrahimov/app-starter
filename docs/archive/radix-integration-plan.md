# Radix Themes integration plan

> **Historical record (archived).** This is the migration plan that moved the UI
> layer from shadcn-lite to Radix Themes. The migration shipped — approval in issue
> #27, merged in #28 — so this plan is kept for rationale only; do not follow its
> steps. The live vocabulary and build workflow are in
> [radix-reference.md](../radix-reference.md) and [radix-workflow.md](../radix-workflow.md).
> Parts below are written in future tense and reference artifacts that were never
> built (notably `docs/radix-migration-manifest.md`).

End-to-end plan to make Radix Themes the default UI layer of this template, with a
single declarative configuration surface, an AI-friendly extension story, and a
rollout designed for the Claude Code `/loop` and `/workflows` features.

Read [radix-reference.md](radix-reference.md) first for the Radix vocabulary this
plan assumes.

## Status and decision

- Decision: adopt Radix Themes (`@radix-ui/themes`) as the default styled UI layer,
  replacing the current shadcn-lite stack (Tailwind v4 + `cva` + `cn` + DTCG
  tokens on five `@radix-ui/react-*` primitives).
- Status (2026-06-20): IMPLEMENTED on branch `claude/clever-shamir-378f9e`. Phases
  0-5 landed and verified green (`tsc`, 141 Vitest tests, Biome, build). Approval
  recorded in issue #27. The bullets and phases below are the original plan; see the
  commit history for the as-built result. Note: removing Tailwind did not shrink the
  CSS bundle (Tailwind had already tree-shaken to ~nothing); the ~693 kB CSS is now
  entirely Radix Themes' own stylesheet.
- Approval gate: this is exactly the kind of change `AGENTS.md` "Approval
  boundaries" reserves for human sign-off -- "adding a UI dependency beyond the
  established set ... a full component kit", "restructuring the design-token set /
  themes", and "architectural conventions not represented by both worked
  examples". It also supersedes the recorded UI direction ("keep the UI layer
  shadcn-lite"). Open a tracking issue and get explicit approval before Phase 1
  writes any code.

## Goals and success criteria

A project author starting from this template can:

1. Restyle the entire app -- accent, gray, radius, scaling, panels, density -- by
   editing one typed config file, or by asking an agent in natural language.
2. Add a new screen or component using documented Radix components and a skill that
   encodes the conventions, with no design decisions left implicit.
3. Trust the result: every change passes the same deterministic gates that exist
   today (`just lint`, `just test`, `just verify`, `just a11y`).

Done when: all current pages (`Home`, `Items`, `Posts`) and the reusable layer run
on Radix Themes; `just verify` and `just a11y` are green; the config surface and
the two skills exist; the docs (`components.md`, `AGENTS.md`, `CLAUDE.md`) describe
Radix Themes instead of shadcn-lite.

## Target architecture

Before vs after:

| Concern | Before (shadcn-lite) | After (Radix Themes) |
| --- | --- | --- |
| Components | hand-written `ui/` + `sections/` | `@radix-ui/themes` components, composed in `sections/` |
| Styling | Tailwind v4 utilities + `cva` + `cn` | Themes props + layout components; Tailwind removed |
| Tokens | DTCG CSS vars in `styles.css` (`:root`/`.dark`/`@theme inline`) | Themes token system; optional custom accent in `theme/accent.css` |
| Config | scattered across component `cva` maps | one `<Theme>` fed by `theme.config.ts` |
| Icons | none | `@radix-ui/react-icons` |
| Dark mode | `.dark` class on `<html>` via `ThemeProvider` + pre-hydration script | unchanged -- Themes reads the same `.dark` class |

Provider tree change (`interface/src/main.tsx`): wrap the app in `<Theme>` inside
the existing `ThemeProvider`, so appearance follows the `.dark` class rather than a
prop.

```tsx
<ThemeProvider>                 {/* keeps the .dark class + localStorage logic */}
  <Theme {...themeConfig}>      {/* themeConfig from theme.config.ts; no appearance prop */}
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
    {import.meta.env.DEV && <ThemePanel />}
  </Theme>
</ThemeProvider>
```

Dark mode is the key compatibility win: the app already toggles `.dark` on `<html>`
(in `ThemeProvider.tsx` and the pre-hydration script in `index.html`), which is one
of the class names Radix Themes recognizes. Do **not** pass
`appearance={resolvedTheme}` -- per Radix docs that causes appearance flashing.
Leave `appearance` at its `inherit`/default and let the class drive it. The
existing `ThemeToggle` keeps working unchanged.

`styles.css` shrinks to: `@import "@radix-ui/themes/styles.css";` then
`@import "./theme/accent.css";` (optional custom brand), plus any app-specific base
rules (safe-area insets for Tauri are worth keeping). The Tailwind import, DTCG
token blocks, `@theme inline`, and the `dark`/`coarse` custom variants are removed.

## The configuration surface (the "easily adjust" core)

A single typed file is the one place to retheme the app. This is what both humans
and the NL skill edit.

`interface/src/theme/theme.config.ts`:

```ts
import type { ThemeProps } from "@radix-ui/themes";

// The single source of truth for global theming. Restyle the whole app here,
// or ask: "set the accent to teal, large radius, 105% scaling".
export const themeConfig = {
  accentColor: "indigo",
  grayColor: "auto",
  panelBackground: "translucent",
  radius: "medium",
  scaling: "100%",
} satisfies Partial<ThemeProps>;
```

Supporting files:

- `interface/src/theme/accent.css` -- optional custom brand color (the 12-step
  override from the Radix custom color tool). Empty by default.
- The status-color mapping (success/pending/destructive/neutral -> Radix hues) is
  documented in `radix-reference.md` and applied as `color=` props at call sites,
  not as global tokens.

Why one declarative file: it makes NL configuration a small, deterministic edit
that the gates can verify, instead of a hunt across component files. It mirrors the
template's existing philosophy -- one OpenAPI contract drives the typed client; one
`<Theme>` drives the visual system.

## Phased rollout

Each phase ends at a green gate so it is resumable and loop/workflow-friendly. Run
the gate listed at each phase boundary; do not advance red.

### Phase 0 -- approval (no code)

Open the tracking issue (AGENTS.md requires it for template-default changes). Get
human approval. Record the issue in the change description.

### Phase 1 -- install and coexist

- `cd interface && bun add @radix-ui/themes @radix-ui/react-icons` (and
  `@radix-ui/colors` only if a custom brand color is needed).
- Add `theme/theme.config.ts`, empty `theme/accent.css`.
- Import `@radix-ui/themes/styles.css` and mount `<Theme>` + dev-only
  `<ThemePanel>` in `main.tsx`. Leave Tailwind in place for now (coexistence).
- Gate: `just lint`, then `cd interface && bun run build`. App still renders via
  the old components; Themes is available.

### Phase 2 -- migrate primitives (`ui/`)

Component-by-component, replace each `ui/*` with a thin wrapper over the Themes
equivalent (mapping table in `radix-reference.md`), keeping the **same export name
and props shape** so pages do not change yet. Update each co-located `*.test.tsx`.
Order: `Button`, `Badge`, `Card`, `Input`, `Checkbox`, `Spinner`, `Dialog`,
`DropdownMenu`, `Tooltip`, `VisuallyHidden`, `EmptyState`, `ErrorState`.

- Gate per component: `cd interface && bunx biome check . && bunx tsc --noEmit && bun run test`.
- This is the unit a `/loop` iteration or a `/workflows` stage operates on.

### Phase 3 -- migrate sections and pages

Re-skin `sections/` (`PageHeader`, `Toolbar`, `FilterBar`, `StatGroup`, `DataList`,
`Field`) with Themes layout + components, then verify `Home`/`Items`/`Posts` render
correctly. `DataList` keeps owning loading/error/empty/data triage; only its
internals change.

- Gate: `just lint`, `cd interface && bun run test`, `just a11y`.

### Phase 4 -- remove the old layer

- Delete DTCG token blocks, Tailwind import, and custom variants from `styles.css`.
- Remove `cva`/`cn` usages (`Button`, `Badge`, `FilterBar`, plus `cn.ts` and its 9
  consumers) and drop `class-variance-authority`, `clsx`, `tailwind-merge`,
  `tailwindcss`, `@tailwindcss/vite`, and the five replaced `@radix-ui/react-*`
  deps from `package.json`.
- Remove the `tailwindcss()` plugin from `interface/vite.config.ts`.
- Gate: `just verify` (full CI set) + `just a11y`.

### Phase 5 -- re-verify and document

- Re-confirm the a11y gate: `e2e/a11y.spec.ts` audits axe on `/`, `/items`,
  `/posts` and checks the three Badge tones for contrast. Update the tone
  assertions to the new Radix `color=` mapping and confirm contrast still passes.
- Update `docs/components.md`, the `add-component` skill, `AGENTS.md` frontend
  conventions, and `CLAUDE.md` quick reminders to describe Radix Themes.
- Ship the `configure-theme` skill (below).

## AI-friendliness design (NLP-driven)

The template is "AI-friendly" because every UI operation reduces to a small,
documented, verifiable edit. Three mechanisms:

1. Declarative single-source config. NL config maps to one edit in
   `theme.config.ts` (or `accent.css` for a custom brand). The vocabulary an agent
   needs is enumerated in `radix-reference.md` (every `<Theme>` prop and its
   values, the component catalog, the status-color mapping).

2. Skills encode the procedure (this template's existing AI-extension mechanism):
   - Update `add-component` so its conventions target Radix Themes: prefer Themes
     components; use `variant`/`size`/`color`/`radius` props; compose `radix-ui`
     primitives + tokens only for the documented gaps (Toast, Accordion, ...);
     keep relative imports, named exports, co-located Vitest, and the a11y rule.
     Keep its bundled `validate-component.sh` gate.
   - Add a `configure-theme` skill: takes a natural-language restyle request, edits
     `theme.config.ts` (and/or `accent.css`), and runs `just lint` +
     `cd interface && bun run build` to verify. Example prompts it must handle:
     "make it teal with large corners", "denser UI" (`scaling="90%"`), "solid
     panels", "switch the gray to slate", "use our brand color #1f6feb".
   - Optionally a machine-readable `interface/src/theme/radix.catalog.json`
     enumerating components, props, and allowed values, so an agent maps NL ->
     props deterministically without re-reading prose.

3. Verifiable gates. The deterministic gates (`just lint`, `just test`,
   `just verify`, `just a11y`) make any AI-produced change checkable. The
   convention-fit sub-agent review already described in the `add-component` skill
   stays as the subjective gate.

Doc/skill updates to land in Phase 5: `docs/components.md` (rewrite for Themes),
`AGENTS.md` step 8 of the resource recipe (Themes components, not the shadcn-lite
catalog), `CLAUDE.md` (UI reminder points at `radix-reference.md`).

## /loop playbook

`/loop` runs a prompt or slash command repeatedly -- on a fixed interval, or
self-paced when no interval is given. The migration is structured so each iteration
is idempotent and ends at a gate, which is exactly what a loop needs.

- Per-component migration loop (self-paced). A migration manifest (a checklist of
  the Phase 2/3 components with a done flag) lets a loop pick the next unmigrated
  component, migrate it, run the component gate, mark it done, and repeat until the
  manifest is empty. Invoke: `/loop migrate the next unmigrated component in
  docs/radix-migration-manifest.md to Radix Themes, then run the component gate`.
- a11y-hardening loop. After Phase 3: `/loop run just a11y, then fix the first axe
  violation` -- keep iterating until axe reports zero violations on all three
  pages.
- Build-watch poll. For a long `bun run build` or a CI run, a fixed-interval
  `/loop` polls status and reports when green.

Requirements that make these loops safe: every iteration must (a) change one
well-scoped unit, (b) end by running a deterministic gate, and (c) record progress
in the manifest so the loop converges and a restart resumes. The phased structure
above satisfies all three.

## /workflows playbook

`/workflows` runs a deterministic multi-agent script (`pipeline`, `parallel`,
`agent`, `phase`). The migration decomposes cleanly into independent, schema-typed
units, so it is a natural fit. Running a workflow requires explicit user opt-in
("use a workflow"); this plan only makes the work workflow-shaped and ships a
ready-to-run script.

Recommended workflows:

- Component migration (pipeline: migrate -> verify per component). One agent
  rewrites each `ui/*`/`sections/*` component on Themes; the next stage runs the
  component gate and reports pass/fail. Pipeline (not barrier) so a fast component
  verifies while a slow one is still being rewritten. A final barrier stage does
  the Phase 4 cleanup (remove Tailwind/cva) once all components pass.
- Page/a11y audit (parallel + adversarial verify). One agent per page audits
  Themes-conformance and a11y; a second independent agent tries to refute each
  finding before it is reported. Catches contrast regressions from the token swap.
- Reference regeneration (parallel -> synthesize). One agent per Radix product
  (Primitives, Themes, Colors, Icons) refreshes its section of
  `radix-reference.md`; a synthesis stage merges and dedups. Useful when Radix
  ships a major version.

Sketch of the migration workflow script (illustrative; refine at implementation):

```js
export const meta = {
  name: "radix-migrate",
  description: "Migrate each UI component to Radix Themes, verify per component",
  phases: [{ title: "Migrate" }, { title: "Verify" }, { title: "Cleanup" }],
};

const COMPONENTS = [
  "ui/Button", "ui/Badge", "ui/Card", "ui/Input", "ui/Checkbox", "ui/Spinner",
  "ui/Dialog", "ui/DropdownMenu", "ui/Tooltip", "ui/VisuallyHidden",
  "ui/EmptyState", "ui/ErrorState",
];

const results = await pipeline(
  COMPONENTS,
  (c) => agent(`Rewrite interface/src/components/${c}.tsx on Radix Themes, keeping the export name and props shape. Update its co-located test. Follow docs/radix-reference.md.`,
    { label: `migrate:${c}`, phase: "Migrate" }),
  (_done, c) => agent(`Run the component gate for ${c}: cd interface && bunx biome check . && bunx tsc --noEmit && bun run test. Report pass/fail and any error.`,
    { label: `verify:${c}`, phase: "Verify", schema: GATE_SCHEMA }),
);

const allGreen = results.filter(Boolean).every((r) => r.passed);
if (allGreen) {
  await agent("Phase 4 cleanup: remove Tailwind import, DTCG tokens, cva/cn, and dead deps; drop the tailwindcss vite plugin. Run just verify.",
    { phase: "Cleanup" });
}
return { migrated: results.filter(Boolean).length, allGreen };
```

## Risks, conflicts, and gates

- Supersedes prior direction. This reverses the recorded "shadcn-lite" UI decision
  and the DTCG-token framing. Confirm the pivot is intended (it was chosen for this
  plan) and update the project's UI-direction notes.
- AGENTS.md approval boundaries. Triggers the UI-dependency, design-token-
  restructure, and architectural-convention gates. Requires an issue + human
  approval (Phase 0).
- Coverage gaps. Themes lacks Toast, Accordion, NavigationMenu, Toolbar,
  Collapsible, ToggleGroup, and Form. These become `radix-ui` primitive +
  token compositions; budget for them when a screen needs one.
- Tailwind removal blast radius. `cva` in 3 files, `cn` in 9, Tailwind import in
  `styles.css` + vite plugin + 2 deps. Contained, but do it as its own phase
  (Phase 4) after components are proven on Themes.
- a11y contrast re-validation. The token swap changes every color. The existing
  axe gate (`e2e/a11y.spec.ts`) is the safety net; re-run it (Phase 5) and update
  the Badge-tone assertions to the Radix `color=` mapping.
- Appearance flash. Never set `<Theme appearance={resolvedTheme}>`; rely on the
  `.dark` class (documented above) to avoid a flash on load.
- Bundle size. Themes adds a styled component library + its CSS. Acceptable for an
  app starter; note it so a size-sensitive project can tree-shake or stay on
  Primitives.
- Desktop (Tauri). Keep the safe-area base rules and the Tauri-aware `baseUrl`
  logic in `api/client.ts` (a hard rule in AGENTS.md). Themes is client-only CSS,
  so the sidecar is unaffected; verify with `just desktop-build` if touched.

## File-by-file change map

| File | Change | Phase |
| --- | --- | --- |
| `interface/package.json` | add Themes/icons(/colors); later remove Tailwind/cva/cn/old primitives | 1, 4 |
| `interface/src/main.tsx` | mount `<Theme>` + dev `<ThemePanel>`; no `appearance` prop | 1 |
| `interface/src/theme/theme.config.ts` | new -- the config surface | 1 |
| `interface/src/theme/accent.css` | new -- optional custom brand color (empty-but-templated) | 1 |
| `interface/src/theme/icons.ts` | new -- enumerable Radix Icons allow-list for agents | 2 |
| `interface/src/styles.css` | swap Tailwind+DTCG for Themes CSS import; keep safe-area base | 1, 4 |
| `interface/src/components/ui/*` | rewrite on Themes (same export/props) | 2 |
| `interface/src/components/sections/*` | re-skin with Themes layout/components | 3 |
| `interface/src/components/ui/cn.ts` | remove after consumers drop it | 4 |
| `interface/src/components/theme/ThemeProvider.tsx` + `ThemeToggle.tsx` | keep (class-based dark mode still drives Themes) | - |
| `interface/index.html` | keep the pre-hydration `.dark` script | - |
| `interface/vite.config.ts` | remove `tailwindcss()` plugin | 4 |
| `interface/e2e/a11y.spec.ts` | update Badge-tone assertions; re-verify contrast | 5 |
| `docs/components.md`, `AGENTS.md`, `CLAUDE.md` | rewrite UI guidance for Themes | 5 |
| `.claude/skills/add-component/` | retarget conventions to Themes | 5 |
| `.claude/skills/configure-theme/` | new NL retheme skill | 5 |
| `docs/radix-migration-manifest.md` | new -- per-component checklist the `/loop` consumes | 1 |

## Decisions (resolved 2026-06-20)

The three open questions were researched against current (June 2026) Radix and
Tailwind docs, and each recommendation was adversarially reviewed through a
technical lens and a template-fit lens. All three are downstream of the Phase 0
approval gate (adopting Radix Themes at all).

### 1. Tailwind: remove it entirely. Confidence: high.

Make Radix Themes the single styling system. Remove `tailwindcss`,
`@tailwindcss/vite`, `class-variance-authority`, `clsx`, `tailwind-merge`, the
Tailwind import + DTCG tokens + `@theme inline` + custom variants from
`styles.css`, and the `tailwindcss()` vite plugin (Phase 4).

Why -- the decisive reason is determinism, not CSS hygiene. The first goal is
"restyle the whole app from one config file / one NL edit". Two styling systems
means color and spacing can live in either `theme.config.ts` or a utility class, so
an NL restyle is no longer one deterministic edit. Coexistence is actually fine in
Tailwind v4 -- a single `@import "@radix-ui/themes/styles.css" layer(components);`
resolves the Preflight-vs-Themes button-reset conflict -- so "it is too brittle" is
not the argument; "it is a second source of truth the template does not need" is.
The blast radius (cva in 3 files; `cn` in 9 consumers + `cn.ts`/`cn.test.ts`) is
removed by Phases 2-4 anyway, so keeping Tailwind would preserve a system whose last
consumers were just deleted.

Acceptance criterion (make it a gate, not an assumption): after the migration, an
NL restyle like "make it teal, denser" must remain ONE deterministic edit to
`theme.config.ts`. If Themes-prop styling instead sprawls across components, the
removal loses the property that justified it.

Watch: the Tauri safe-area `body` rule uses
`@apply bg-background text-foreground antialiased`; those utilities vanish with
Tailwind. Re-express background/text via the `<Theme>` root (it paints the page) and
keep only the `env(safe-area-inset-*)` padding as plain CSS.

### 2. Icons: standardize on Radix Icons + an enumerable allow-list. Confidence: medium-high.

Once Themes lands, add `@radix-ui/react-icons` as the default set and ship a small
enumerable allow-list (`interface/src/theme/icons.ts`) re-exporting the ~8-12 glyphs
the template uses (sun/moon, sort caret, dropdown chevron, checkbox check,
success/pending/error, empty/error states). `lucide-react` is the approval-gated
escape hatch for projects that outgrow it -- swap entirely, never mix the two.

Why -- the 15x15 currentColor set drops into Themes components with no size/color
reconciliation, keeps the stack under one vendor, and (most important for the AI
story) a fixed, enumerable allow-list lets an agent map a natural-language need to
an exact import deterministically; lucide's ~1500 icons invite hallucinated names.

Two corrections from the adversarial review, both real:

- The package is NOT frozen. Stable `latest` is 1.3.2 (Nov 2024), but an active
  2.0.0 release-candidate line exists under the `@next` tag (newest Apr 2026). Pin
  `~1.3.2` and do NOT install `@next` / `2.0.0-rc` until its breaking changes (it
  already changes the React peer range) are vetted.
- This is the template's first-ever icon runtime dependency (today the Spinner is
  CSS, the checkbox check is an inline `<svg>`). It is genuinely approval-gated and
  only justified once Themes lands; until then, inline SVG stays the minimal,
  hallucination-proof default. The allow-list is the durable win and is worth
  keeping even in front of an inline-SVG map.

Doc nit corrected in the reference: the set is ~332 icons, not ~300.

### 3. Custom brand color: ship `accent.css` empty-but-templated. Confidence: high.

Ship `interface/src/theme/accent.css` empty by default -- a commented, paste-ready
skeleton carrying the exact override recipe -- not a live example palette. The
default app stays on a built-in hue via `accentColor`; a custom brand is one
uncomment/paste away.

Why -- a live palette is ~60-100 lines of generated CSS (12 steps + alpha + P3, x2
for light/dark) that is noise for the ~95% of projects on a built-in hue, must pass
`just a11y` contrast on every Radix bump, and is exactly the drift-prone generated
artifact the template's minimalism standard rejects. The worked example for accent
already exists: the zero-CSS `accentColor` prop. Empty trades
"continuously-tested-but-bloated" for "untested-until-used-but-zero-surface", and
the gates fire on the author's real brand when they use it.

The `configure-theme` skill owns the recipe so it runs at use-time: generate a
12-step scale at radix-ui.com/colors/custom, rename it to the hue currently named in
`theme.config.ts` (read that value -- do not assume `indigo`), paste the light block
into a `.radix-themes`-scoped override and the dark block into a `.dark` override
(matching the existing class toggle), then run `just lint` + build + `just a11y`.
The six version-sensitive details (exact `--accent-*` token set, generator
selectors, full-scale vs step-9-only, P3 `@supports`, the `.radix-themes` wrapper)
live in the skill procedure, verified at use-time.

Synergy with decision 1: removing Tailwind also removes the Preflight import-order
hazard that would otherwise threaten the accent override, so decisions 1 and 3
reinforce each other.
