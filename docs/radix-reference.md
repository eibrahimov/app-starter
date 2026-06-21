# Radix UI reference

A documented map of the Radix ecosystem, scoped to this template. It is the
vocabulary that humans and AI agents read before configuring or extending the UI.
Pair it with [radix-integration-plan.md](radix-integration-plan.md), which turns
this map into an end-to-end migration.

Radix is not one library. It is four products that integrate very differently.
Knowing which product a request belongs to is the first decision in every UI task.

Status: Radix Themes is the shipped default UI layer (the migration described in
[radix-integration-plan.md](radix-integration-plan.md) landed via #28). This
reference and the shipped code agree; the plan is now historical context for how
it got here. For the end-to-end "build a new app from a natural-language
description" lifecycle that uses everything below, see
[radix-workflow.md](radix-workflow.md).

## The four Radix products

| Product | Package | What it is | Role in this template |
| --- | --- | --- | --- |
| Themes | `@radix-ui/themes` | A fully styled, themeable component library (buttons, cards, dialogs, layout, typography) with a single `<Theme>` config provider. | The default UI layer. Source of components and styling. |
| Primitives | `radix-ui` (unified) or `@radix-ui/react-*` | Unstyled, accessible behavior primitives (focus, keyboard, ARIA) with no visuals. | Composed only where Themes has a gap (see the gaps section). Themes is built on top of these. |
| Colors | `@radix-ui/colors` | 12-step accessible color scales (light, dark, and alpha) for ~30 hues. | The palette Themes draws accent and gray from. Edited only for a custom brand color. |
| Icons | `@radix-ui/react-icons` | ~332 crisp 15x15 icons as React components. | The default icon set (pin `~1.3.2`). Swap for a larger set when it is not enough. |

The previous UI layer ("shadcn-lite": hand-written components on a handful of
`@radix-ui/react-*` primitives, styled with Tailwind v4 + `cva` + `cn` + DTCG
tokens) is replaced by Themes. See the integration plan for the migration and the
approval gate.

## Packages and versions

Shipped pins in `interface/package.json`: `@radix-ui/themes` `^3.3.0` and
`@radix-ui/react-icons` `^1.3.2` (the caret stays within the 1.x line, below the
unvetted 2.0 icon release — see Radix Icons below). `@radix-ui/colors` and the
unstyled `radix-ui` primitives are added on demand (`cd interface && bun add ...`)
when a custom brand color or a documented Themes gap needs them. Confirm exact
versions against npm before bumping.

| Package | Purpose | Replaces |
| --- | --- | --- |
| `@radix-ui/themes` | Styled components + `<Theme>` provider + tokens | the `ui/` primitives, `cva`, the Tailwind styling layer |
| `@radix-ui/react-icons` | Icon set | (new) the app had no icon set |
| `@radix-ui/colors` | Custom brand-color scales (optional) | DTCG color tokens in `styles.css` |
| `radix-ui` | Unstyled primitives for Themes gaps (Toast, Accordion, ...) | the individual `@radix-ui/react-*` deps |

Removed by the migration: `@radix-ui/react-checkbox`, `-dialog`,
`-dropdown-menu`, `-tooltip`, `-visually-hidden` (Themes bundles equivalents),
`class-variance-authority`, `clsx`, `tailwind-merge`, `tailwindcss`,
`@tailwindcss/vite` (Tailwind removal is a documented escape-hatch decision, not
mandatory -- see the plan).

## The `<Theme>` provider: the global config surface

One element configures the entire app. These props are the canonical vocabulary an
NL prompt maps to ("make the accent teal, large radius, translucent panels" ->
`accentColor="teal" radius="large" panelBackground="translucent"`).

| Prop | Values | Default | Effect |
| --- | --- | --- | --- |
| `appearance` | `light`, `dark`, `inherit` | `light` | Light/dark mode. In this template, leave it `inherit` and let the `.dark` class on `<html>` drive it (see dark mode below). |
| `accentColor` | about two dozen named hues (~25; verify the exact list against current docs -- e.g. `indigo`, `blue`, `teal`, `jade`, `grass`, `crimson`, `tomato`, `amber`, `iris`, ...) | `indigo` | The primary brand color used by interactive elements. |
| `grayColor` | `auto`, `gray`, `mauve`, `slate`, `sage`, `olive`, `sand` | `auto` | The neutral scale. `auto` picks a gray that harmonizes with the accent. |
| `panelBackground` | `solid`, `translucent` | `translucent` | Whether panels/menus are opaque or blurred-translucent. |
| `radius` | `none`, `small`, `medium`, `large`, `full` | `medium` | Global corner radius. |
| `scaling` | `90%`, `95%`, `100%`, `105%`, `110%` | `100%` | Global size multiplier (density). |
| `hasBackground` | boolean | `true` | Whether the root paints a page background. |

Nested `<Theme>` elements inherit and override per subtree (e.g. a `crimson`
section inside an `indigo` app). This is how to theme one region without touching
global config.

### `<ThemePanel>`

A drop-in floating panel (`<ThemePanel />`, toggled with the `T` key) that lets a
human pick every prop above at runtime and click "Copy Theme" to get the exact
`<Theme ...>` JSX. It is the manual counterpart to the NL `configure-theme` skill:
the panel writes the same JSX a prompt would. Ship it behind a dev-only flag.

## Component catalog (Radix Themes)

> This prose catalog has a machine-readable companion at
> [`interface/src/theme/radix.catalog.json`](../interface/src/theme/radix.catalog.json):
> the same prop values, components, status colors, icons, and gaps as JSON, for
> agents (and non-Claude LLM tooling) that prefer a structured lookup. A guard
> test (`radix.catalog.test.ts`) keeps it in sync with `icons.ts` and
> `theme.config.ts`, so it cannot silently drift from this doc.

Grouped by purpose. Bold = the app already needs an equivalent today (mapped from
the current `ui/`+`sections/` layer).

- Layout (replace Tailwind utility wrappers): **Box**, **Flex**, **Grid**,
  **Container**, **Section**, Inset, AspectRatio.
- Typography: **Text**, **Heading**, Blockquote, Code, Em, Kbd, Link, Quote,
  Strong, Separator.
- Forms and inputs: **TextField**, **TextArea**, **Checkbox**, CheckboxGroup,
  CheckboxCards, RadioGroup, RadioCards, **Select**, Slider, **Switch**,
  SegmentedControl.
- Buttons and actions: **Button**, **IconButton**, **DropdownMenu**, ContextMenu.
- Data display: **Card**, **Badge**, Avatar, **Table**, DataList, **Callout**,
  Progress, **Spinner**, **Skeleton**.
- Overlays and feedback: **Dialog**, AlertDialog, Popover, HoverCard, **Tooltip**.
- Navigation: TabNav, Tabs, Link.
- Utilities: **VisuallyHidden**, Portal, Reset, Theme, ThemePanel.

Mapping from the current layer:

| Current component | Radix Themes target |
| --- | --- |
| `ui/Button` (cva variants) | `Button` / `IconButton` (`variant`, `size`, `color`) |
| `ui/Badge` (tone cva) | `Badge` (`color`, `variant`) |
| `ui/Card` | `Card` |
| `ui/Input` | `TextField.Root` (+ `TextField.Slot`) |
| `ui/Checkbox` | `Checkbox` |
| `ui/Dialog` | `Dialog.*` (or `AlertDialog.*` for confirms) |
| `ui/DropdownMenu` | `DropdownMenu.*` |
| `ui/Tooltip` | `Tooltip` |
| `ui/Spinner` | `Spinner` |
| `ui/EmptyState`, `ui/ErrorState` | compose `Callout` + `Flex` + `Text` |
| `ui/VisuallyHidden` | `VisuallyHidden` |
| `sections/PageHeader` | compose `Flex`/`Heading`/`Text` |
| `sections/Toolbar`, `FilterBar` | compose `Flex` + `Button`/`SegmentedControl` |
| `sections/StatGroup` | compose `Grid` + `Card` |
| `sections/DataList` (triage owner) | keep the component; re-skin internals with Themes (`Spinner`, `Callout`, `Card`) |
| `sections/Field` (a11y form row) | keep; re-skin with `Text` label + Themes input |

## Styling vocabulary (per-component props)

Themes components are configured by props, not utility classes. The common axes:

- `variant`: `solid`, `soft`, `surface`, `outline`, `ghost` (exact set varies per
  component; Button/Badge/Card support most).
- `size`: `1`, `2`, `3`, `4` (numeric scale; not every component has all four).
- `color`: any Radix Colors hue, overriding the theme accent for that element
  (e.g. a `red` destructive button, `amber` pending badge, `grass` success).
- `highContrast`: boolean, for stronger contrast variants.
- `radius`: per-element override of the global radius.

Semantic status mapping for this app (replaces the old `text-success` /
`text-warning` / `text-destructive` tokens and the Badge tones the a11y test
audits): success -> `color="grass"` (or `jade`), pending/draft -> `color="amber"`,
destructive -> `color="red"` (or `tomato`), neutral/archived -> `color="gray"`.

## Layout props (the Tailwind-utility replacement)

`Box`, `Flex`, `Grid`, `Container`, `Section` accept a constrained set of spacing
and layout props on the theme scale, so most Tailwind utilities disappear:

- spacing: `p`, `px`, `py`, `pt/pr/pb/pl`, `m`, `mx`, ... (scale `1`-`9`).
- flex/grid: `direction`, `align`, `justify`, `gap`, `wrap`, `columns`, `rows`,
  `flexGrow`, `flexShrink`.
- sizing: `width`, `minWidth`, `maxWidth`, `height` (CSS values or scale).
- responsive object syntax for every prop:
  `<Flex direction={{ initial: "column", md: "row" }} gap={{ initial: "2", md: "4" }}>`.
  Breakpoints: `initial`, `xs`, `sm`, `md`, `lg`, `xl`.

Touch targets (the app's `coarse:min-h-11` rule) are satisfied by using
`size="3"`+ on interactive components; verify with the existing a11y gate.

## Radix Colors: the 12-step scale system

Each hue is a 12-step scale with a defined role per step. Themes consumes these
automatically; you rarely touch them unless adding a custom brand color.

| Steps | Role |
| --- | --- |
| 1-2 | App and subtle backgrounds |
| 3-5 | Component backgrounds (normal, hover, active) |
| 6-8 | Borders and separators (subtle, normal, hover) |
| 9-10 | Solid backgrounds (9 is the pure brand color; 10 is its hover) |
| 11 | Low-contrast text |
| 12 | High-contrast text |

Every hue also has an **alpha** scale (`-a1`..`-a12`) for layering over arbitrary
backgrounds. Dark mode scales are separate files and are selected automatically by
the `.dark` class -- you do not hand-tune dark values.

Themes exposes the active accent and gray as CSS variables you can read in custom
CSS: `--accent-1`..`--accent-12`, `--accent-a1`..`--accent-a12`, `--accent-9`
(solid), `--accent-contrast`, `--accent-surface`, `--accent-indicator`,
`--accent-track`; and the gray equivalents `--gray-1`..`--gray-12`,
`--gray-a1`..`--gray-a12`. (Confirm the exact variable names against
radix-ui.com/themes/docs/theme/color during implementation.)

## Custom accent / brand color

Two paths, in order of NL-friendliness:

1. Built-in hue (recommended default). Set `accentColor` to one of the ~29 Radix
   hues. Fully NL-driven, zero CSS, dark mode handled. Covers most brands.
2. Custom brand hex. Generate a 12-step scale with the Radix custom color tool
   (radix-ui.com/colors/custom), drop it into a dedicated file
   (`interface/src/theme/accent.css`), and override the `--accent-*` variables
   scoped to `.radix-themes`. Keep this in one file so a brand change is one edit.
   This path is the escape hatch; verify the exact override recipe against current
   Radix docs, because custom-color wiring is the most version-sensitive detail.

## Radix Icons

```tsx
import { GearIcon, SunIcon, MoonIcon, CaretSortIcon } from "@radix-ui/react-icons";

<GearIcon />                 {/* inherits currentColor and 15x15 size */}
<SunIcon color="#f59e0b" />  {/* explicit color */}
<MoonIcon width={20} height={20} /> {/* resize */}
```

Icons inherit `currentColor` and a 15x15 footprint, so they sit naturally inside
Themes `Button`/`IconButton`/`Text`. The set is ~332 icons (navigation, layout,
status, editing). Pin `~1.3.2`: stable `latest` is 1.3.2 (Nov 2024); a 2.0.0
release-candidate line exists under the `@next` tag (do not install it until its
breaking changes -- including a changed React peer range -- are vetted). Expose the
handful the template actually uses through an enumerable allow-list
(`interface/src/theme/icons.ts`) so agents pick from a closed set deterministically.
If a project needs a broader set, that is a gated dependency decision (AGENTS.md):
swap entirely to e.g. `lucide-react` -- document the swap, never mix two icon
libraries.

## What Radix Themes does NOT include

Themes is intentionally curated. These common components are not in Themes; build
them by composing unstyled primitives from `radix-ui` with Theme tokens (use
`var(--accent-9)`, `Text`, `Flex`, etc. so they match the theme):

- Toast / Toaster (notifications)
- Accordion / Collapsible
- NavigationMenu / Menubar / Toolbar
- ToggleGroup / Toggle
- Form (use native forms + Themes inputs; the app's `Field` already does this)

When a request needs one of these, the task is a Primitives-composition task, not a
Themes-config task. The `add-component` skill routes this decision.

## Reference links

- Themes: https://www.radix-ui.com/themes/docs/overview/getting-started
- Themes color/theming: https://www.radix-ui.com/themes/docs/theme/color
- Themes dark mode: https://www.radix-ui.com/themes/docs/theme/dark-mode
- Colors: https://www.radix-ui.com/colors and custom: https://www.radix-ui.com/colors/custom
- Primitives: https://www.radix-ui.com/primitives
- Icons: https://www.radix-ui.com/icons
