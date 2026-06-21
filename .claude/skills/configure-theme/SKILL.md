---
name: configure-theme
description: >-
  Apply a natural-language restyle request to this app-starter's Radix Themes UI
  by editing the single global config file (or the custom-accent escape hatch),
  then verifying with the deterministic gates. Use when asked to change the
  accent/theme color, make the UI denser or larger, get rounder or sharper
  corners, use a brand color by hex (e.g. "use our brand color #1f6feb"), switch
  the gray/neutral, make panels translucent or solid, or otherwise retheme the
  whole app. Encodes the two paths (built-in <Theme> prop vs. a custom 12-step
  brand scale in accent.css) and ends with the build + a11y gates.
---

# Configure the theme (natural-language restyle)

This skill turns a restyle request into a **small, deterministic edit** plus a
gate run. The whole point of the Radix Themes layer is that global styling lives
in one declarative place, so an NL request maps to a single config edit -- not a
hunt across components.

Authoritative references -- read before mapping a request to props:

- [`docs/radix-reference.md`](../../../docs/radix-reference.md) -> the `<Theme>`
  prop table (every prop + allowed values), the status-color mapping, and the
  custom-accent recipe.
- [`docs/radix-integration-plan.md`](../../../docs/radix-integration-plan.md) ->
  the resolved decisions (notably: `accent.css` ships empty; the skill owns the
  custom-color recipe and runs it at use-time).
- [`interface/src/theme/radix.catalog.json`](../../../interface/src/theme/radix.catalog.json)
  -> the `<Theme>` prop value sets as parseable JSON (`theme.props.*.values`);
  a guard test keeps it in sync with `theme.config.ts`.

The single global config surface is
[`interface/src/theme/theme.config.ts`](../../../interface/src/theme/theme.config.ts)
(fed into `<Theme>`). A custom brand hex lives only in
[`interface/src/theme/accent.css`](../../../interface/src/theme/accent.css).

## Read these invariants first (this is where retheming drifts)

1. **One config file for built-in changes.** Hue, gray, panel style, radius, and
   density are all `<Theme>` props in `theme.config.ts`. Never restyle by editing
   component files or adding utility classes -- that reintroduces a second source
   of truth and breaks the "one NL edit" property.
2. **Use only documented prop values.** Each prop has a closed value set (below,
   from `docs/radix-reference.md`). Do not invent values; a typo fails `tsc`.
3. **Never add an `appearance` prop.** Light/dark is driven by the `.dark` class
   on `<html>` (ThemeProvider + the pre-hydration script). Passing `appearance`
   causes a flash on load. A "dark mode" request is about the toggle, not config.
4. **Custom hex is the escape hatch, not the default.** Prefer a built-in hue
   when the request is a color *name*. Only touch `accent.css` when the request
   is a specific *hex* no built-in hue matches.
5. **Rename to the CURRENT accent hue, not `indigo`.** The custom scale must
   override the hue named by `accentColor` in `theme.config.ts`. Read that value;
   do not assume `indigo`.
6. **Status colors are call-site `color=` props, not global config.** A request
   to recolor success/pending/destructive states maps to the status mapping in
   `docs/radix-reference.md` (`grass`/`amber`/`red`), applied per element -- it is
   an `add-component` task, not a `theme.config.ts` edit.

## Allowed values (built-in props, per `docs/radix-reference.md`)

| Prop | Allowed values | What the request sounds like |
| --- | --- | --- |
| `accentColor` | one of the ~25 named Radix hues (`indigo`, `blue`, `teal`, `jade`, `grass`, `crimson`, `tomato`, `amber`, `iris`, ...; verify the full list against current docs) | "make it teal", "use a green accent" |
| `grayColor` | `auto`, `gray`, `mauve`, `slate`, `sage`, `olive`, `sand` | "switch the gray to slate", "warmer neutrals" |
| `panelBackground` | `solid`, `translucent` | "solid panels", "translucent/blurred menus" |
| `radius` | `none`, `small`, `medium`, `large`, `full` | "rounder corners", "sharp/square corners" |
| `scaling` | `90%`, `95%`, `100%`, `105%`, `110%` | "denser UI" (smaller), "make it bigger/roomier" |

A request that names a *color* maps to `accentColor`; a *hex* triggers the custom
path below.

## Procedure

### Path A -- built-in change (hue / gray / panel / radius / density)

The default and overwhelmingly common case -- zero CSS, dark mode handled.

1. Map the request to exactly one prop and one allowed value from the table above
   (cross-check `docs/radix-reference.md` if unsure).
2. Edit `interface/src/theme/theme.config.ts`: change that single property in the
   `themeConfig` object. Leave the others untouched. Do not add `appearance`.
3. Verify (mandatory gates, below).

### Path B -- custom brand hex (`accent.css`)

Only when the request gives a specific hex (e.g. "use our brand color #1f6feb")
that no built-in hue matches. This follows the recipe documented in
`accent.css`'s header comment.

1. **Read the current accent hue** from `theme.config.ts` (the value of
   `accentColor`). All renames target THIS hue, not `indigo`.
2. **Generate the scale** at <https://www.radix-ui.com/colors/custom>: paste the
   hex. It emits a 12-step light scale + alpha (`--custom-1..12`,
   `--custom-a1..a12`), a dark scale + alpha, and wide-gamut
   `color(display-p3 ...)` values inside `@supports` blocks.
3. **Rename** every generated `--custom-*` variable to the current accent hue
   (e.g. with `accentColor="indigo"`: `--custom-1` -> `--indigo-1` ...
   `--custom-12`, and `--custom-a1` -> `--indigo-a1` ... `--custom-a12`).
4. **Scope and paste into `accent.css`** (uncomment the skeleton): the LIGHT
   scale under `.radix-themes`; the DARK scale under `.dark, .dark .radix-themes`
   (`.dark` is the class the ThemeProvider toggles on `<html>`).
5. **Keep the P3 blocks.** Preserve both
   `@supports (color: color(display-p3 1 1 1))` / `@media (color-gamut: p3)`
   wrappers (light and dark) so wide-gamut displays get the richer scale.
6. Leave `accentColor` in `theme.config.ts` set to the same hue you renamed to --
   the override replaces that hue's scale in place.
7. Verify (gates, below) -- including `just a11y` for contrast.

## Validate before handoff (mandatory)

Run from `interface/`:

```sh
cd interface
bunx tsc --noEmit     # catches an invalid prop value or a typo'd hue
bun run build         # confirms Themes/CSS compile
```

For a custom palette (Path B), also confirm contrast did not regress -- the new
scale changes every accent color, so the a11y gate is the safety net:

```sh
just a11y
```

Report the commands you ran and their results. Do not hand off a red theme.

## Stop and get human approval before

These exceed a config edit and are gated by `AGENTS.md` "Approval boundaries":

- adding a UI dependency or a new theming mechanism beyond `<Theme>` props +
  `accent.css` (e.g. a second styling system, a runtime theme switcher beyond the
  shipped dev-only `<ThemePanel>`);
- introducing per-component global style overrides or new design tokens outside
  the documented `<Theme>` surface;
- swapping the icon set (`@radix-ui/react-icons` -> `lucide-react`) -- that is the
  `add-component` / icon-allow-list path, not a theme config change.
