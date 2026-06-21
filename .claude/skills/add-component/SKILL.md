---
name: add-component
description: >-
  Add a frontend UI primitive, composite section, or data hook to this
  app-starter template, following the components/ui, components/sections, and
  hooks layers and the Items/Posts worked examples. Use when asked to add a
  component, button, input, badge, card, dialog, dropdown, tooltip, list/table
  section, page header, filter bar, or a data-fetching hook, or to style a piece
  of UI with Radix Themes. Encodes the frontend conventions (Radix Themes
  components + props, relative imports, no barrels, typed API access) and ends
  with a validation script that runs the deterministic gates.
---

# Add a frontend component, section, or hook

This skill is the **operational procedure** for extending the reusable frontend
layer, which is built on **Radix Themes** (`@radix-ui/themes`). Copy/paste-level
detail lives in the canonical docs — keep this skill in sync with them rather than
restating everything:

- [`docs/radix-reference.md`](../../../docs/radix-reference.md) — the Radix
  vocabulary: component catalog, the current-layer → Themes mapping, `<Theme>`
  props, the styling/layout props, status-color mapping, and the documented gaps.
- [`docs/components.md`](../../../docs/components.md) (human-facing, with the full
  catalog and snippets)
- [`docs/radix-workflow.md`](../../../docs/radix-workflow.md) — where this skill
  sits in the end-to-end build lifecycle (the Integrate stage).
- [`interface/src/theme/radix.catalog.json`](../../../interface/src/theme/radix.catalog.json)
  — the machine-readable component/prop/icon/gap catalog (the same vocabulary as
  the reference, parseable; kept in sync by `radix.catalog.test.ts`).
- [`AGENTS.md`](../../../AGENTS.md) -> frontend conventions (agent-facing)

The layer has three tiers. Copy the shape of what already exists:

- **`interface/src/components/ui/`** — primitives (atoms) that wrap or re-export a
  Radix Themes component with this app's defaults: `Button`, `Input`
  (`TextField`), `Card`, `Badge`, `Spinner`, `EmptyState`, `ErrorState`,
  `Checkbox`, `Dialog`, `DropdownMenu`, `Tooltip`, `VisuallyHidden`. Reference:
  `Button.tsx`.
- **`interface/src/components/sections/`** — composite blocks that compose
  primitives with Themes layout (`Box`/`Flex`/`Grid`): `PageHeader`, `Toolbar`,
  `FilterBar`, `StatGroup`, `DataList`, and the accessible-form `Field`.
  Reference: `DataList.tsx` (owns loading/error/empty/data triage).
- **`interface/src/hooks/`** — typed data hooks over openapi-fetch + React Query:
  `useApiQuery`, `useApiMutation`, and the `useResource` convenience layer.

The single global theming surface is `interface/src/theme/theme.config.ts` (fed
into `<Theme>`); the custom brand color lives in `interface/src/theme/accent.css`;
the enumerable icon allow-list is `interface/src/theme/icons.ts`.

The canonical worked example of all three composed together is the refactored
`interface/src/pages/Items.tsx`.

## Read these invariants first (this is where components drift)

1. **Reach for a Radix Themes component first.** Before writing anything, check
   the catalog + the current-layer → Themes mapping in
   [`docs/radix-reference.md`](../../../docs/radix-reference.md). Most needs are a
   `Button`, `Badge`, `Card`, `TextField`, `Checkbox`, `Select`, `Switch`,
   `Dialog`, `DropdownMenu`, `Tooltip`, `Callout`, `Table`, `Spinner`, or
   `Skeleton` — use it instead of building from scratch.
2. **Style with Themes props, not utility classes.** Configure components with
   `variant` (`solid`/`soft`/`surface`/`outline`/`ghost`), `size` (`1`-`4`),
   `color` (any Radix hue), `radius`, and `highContrast` — never Tailwind
   utilities, `cva`, `cn`, or `twMerge`. Status colors map per the reference:
   success -> `color="grass"` (or `jade`), pending/draft -> `color="amber"`,
   destructive -> `color="red"` (or `tomato`), neutral/archived -> `color="gray"`.
   Don't hard-code a hex color; let the active accent/gray and the `.dark` class
   drive theming. Global theme changes go in `interface/src/theme/theme.config.ts`
   (and a custom brand hue in `interface/src/theme/accent.css`), never inline.
3. **Lay out with Themes layout components + the spacing scale.** Use `Box`,
   `Flex`, `Grid`, `Container`, and `Section` with their layout props
   (`direction`, `align`, `justify`, `gap`, `columns`, `p`/`px`/`m`/...) on the
   `1`-`9` spacing scale — including the responsive object syntax
   (`gap={{ initial: "2", md: "4" }}`). These replace Tailwind utility wrappers.
4. **Relative imports, no `@/` aliases, no barrel/index files.** Import each
   component by its path (`../components/ui/Button`). Do not add an
   `index.ts` re-export — the template stays explicit.
5. **Named function exports.** `export function Foo()` — no default exports.
6. **Every `<button>` needs an explicit `type=`.** Prefer the `Button` primitive
   (it defaults to `type="button"`); pass `type="submit"` only for real form
   submits.
7. **For a documented Themes gap, compose `radix-ui` primitives with Theme
   tokens.** Themes does not ship Toast, Accordion, NavigationMenu, Toolbar,
   Collapsible, ToggleGroup, or a Form abstraction (see the gaps section of the
   reference). Build these from the unstyled `radix-ui` primitives and style them
   with Theme tokens — `var(--accent-9)` and friends for color, `Text`/`Flex`/
   `Box` for type and layout — so they match the active theme. Everything else is
   a Themes-config task, not a primitives-composition task.
8. **Use icons from the allow-list.** Import only from
   `interface/src/theme/icons.ts` (the enumerable Radix Icons allow-list). Don't
   reach into `@radix-ui/react-icons` directly; add the glyph to the allow-list
   if it's missing. Broadening past Radix Icons (e.g. swapping to `lucide-react`)
   is a gated dependency decision.
9. **Data access only through the hooks / the typed `api` client.** Never call
   raw `fetch`. Query/mutation hooks import `api` from `../api/client`; never
   hand-edit `interface/src/api/schema.d.ts` (it is generated by `just typegen`).
10. **Co-locate a Vitest test** (`Name.test.tsx`) that asserts on **behaviour and
   visible output** — accessible role/name, rendered text, callback fires — not on
   class strings (Themes owns the classes). Pure primitives/sections use `render`;
   hooks and anything reading the API use `renderWithClient` / `withClient` from
   `../test-utils` and mock `../api/client`.
11. **Accessible + touch-ready by default.** Every interactive element needs an
   accessible name (text, `aria-label`, or a `VisuallyHidden` / `Field` label).
   Themes supplies the focus rings; keep them. Size for touch with `size="3"`+ on
   interactive components (the reference notes this satisfies the ≥44px target).
   Add an `axe` assertion for non-trivial components (see `src/a11y.test.tsx`);
   the Playwright smoke (`just a11y`) covers page-level landmarks + contrast.

## Procedure

1. **Decide the tier.** Atom with no children-of-its-own-kind -> `ui/`. A block
   that composes primitives and encodes a page layout -> `sections/`. Data
   fetching/mutation logic -> `hooks/`.
2. **Map the request to Radix Themes first** (invariant 1). If a Themes component
   covers it, no install is needed — `@radix-ui/themes` is already a dependency.
   Only when the request hits a documented **gap** (invariant 7) do you reach for
   the unstyled `radix-ui` primitives; confirm they resolve cleanly under React 19.
3. **Create the file** from the nearest existing sibling. Type the props with a
   real interface; for a thin primitive wrapper, extend the Themes component's own
   props and spread `...props` through so callers can pass Themes props (`variant`,
   `size`, `color`, ...), `onClick`, etc.
4. **Co-locate `Name.test.tsx`** (invariant 10) asserting on behaviour, not class
   strings. For a `DataList`-style section, drive a fabricated
   `UseQueryResult`-shaped object — no network needed.
5. **Wire it into a page** if the task calls for it, mirroring the `Items.tsx`
   refactor (PageHeader + Toolbar + DataList + Card/Checkbox/Button), laid out with
   Themes `Box`/`Flex`/`Grid`.
6. **If you add a new top-level source folder**, extend the Vitest coverage
   `include` globs in `interface/vite.config.ts`.

## Validate before handoff (mandatory)

Run the bundled script from anywhere in the repo — it runs the frontend gates
(Biome, tsc, Vitest) and greps the new files for the two frontend footguns
(`@/` imports and `<button>` without `type=`):

```sh
.claude/skills/add-component/scripts/validate-component.sh
```

Then run the full CI set before declaring done:

```sh
just verify
```

Note: adding components does **not** touch the OpenAPI contract, so
`just check-typegen` is a no-op (no `schema.d.ts` change) unless you also added a
resource. Report the commands you ran and their results; do not hand off a red
component.

## Sub-agent review (subjective gate)

The script and `just verify` cover the **deterministic** gates; they cannot judge
convention fit. After gates are green, spawn a fresh-context review sub-agent
(Task tool, `general-purpose`) that did NOT see this skill, give it the diff plus
`Button.tsx`, `DataList.tsx`, and the `Items.tsx` refactor as references, and ask
it to confirm:

- a Radix Themes component is used where one fits; styling is via Themes props
  (`variant`/`size`/`color`/`radius`/`highContrast`) and layout via
  `Box`/`Flex`/`Grid` on the `1`-`9` scale — no Tailwind utilities, `cva`, `cn`,
  or hard-coded hex colors; status colors follow the reference mapping;
- a documented Themes gap (Toast/Accordion/NavigationMenu/Toolbar/Collapsible/
  ToggleGroup/Form) is composed from `radix-ui` primitives + Theme tokens, not
  reinvented;
- icons come from the `interface/src/theme/icons.ts` allow-list;
- accessible name + Themes focus rings + adequate touch size (`size="3"`+) on
  interactive elements;
- relative imports, no `@/` alias, no barrel/index file added;
- named function export; props typed with an interface; `...props` forwarded;
- data access only via the hooks / typed `api` client, never raw `fetch`;
- a co-located Vitest test exists and asserts on behaviour/visible output, not
  class strings.

Incorporate its findings before handoff.

## Stop and get human approval before

These exceed the established pattern and are gated by `AGENTS.md` "Approval
boundaries":

- adding a UI dependency beyond the established set (`@radix-ui/themes`,
  `radix-ui` primitives, `@radix-ui/react-icons`, `@radix-ui/colors`,
  `jest-axe` / `@axe-core/playwright`) — e.g. a full alternative component kit, a
  second icon set (e.g. `lucide-react`), a CSS-in-JS lib, or re-introducing
  Tailwind / `cva` / `clsx` / `tailwind-merge`;
- restructuring the global theme surface beyond
  `interface/src/theme/theme.config.ts` and the custom-accent escape hatch in
  `interface/src/theme/accent.css` (extra `<Theme>` config layers, new brand
  scales);
- adding a barrel/index re-export or `@/` path alias convention;
- a global state manager beyond React Query + local `useState`.
