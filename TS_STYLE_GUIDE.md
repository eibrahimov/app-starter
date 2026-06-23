# TypeScript Style Guide

## 1. Scope and how to use

This guide describes the TypeScript and frontend conventions actually shipped
in `interface/` for App Starter, an MIT-licensed full-stack template. It is the
companion to `RUST_STYLE_GUIDE.md` (backend) and to `AGENTS.md`, which holds the
project's validation commands and the resource recipe; read those first for the
full picture. Everything below reflects how the existing `interface/` code is
written, formatted, typed, and tested. When you add a page, a query, or a
component, match these patterns rather than introducing new ones. Where a rule
is a universal best practice rather than a repo-specific choice, it is still
written as a contributor rule you are expected to follow.

## 2. Toolchain and gates

Use **Bun** for everything in `interface/`: it is the package manager and the
script runner. Never use `npm`, `pnpm`, or `yarn`. Use `bun` and `bunx` only.

The scripts in `interface/package.json` are the canonical entry points:

- `bun run dev` -- Vite dev server on `:5173` (proxies `/api` to the backend on
  `:8080`).
- `bun run build` -- production build (embedded into the Rust binary).
- `bun run preview` -- serve the production build locally.
- `bun run lint` -- `biome check .` (format + lint, read-only).
- `bun run format` -- `biome check --write .` (apply fixes).
- `bun run typecheck` -- `tsc --noEmit`.
- `bun run test` -- `vitest run`.
- `bun run test:coverage` -- `vitest run --coverage`.

These are the gates a change must pass before it merges:

1. **Biome** -- run `bun run lint` locally and keep it clean. The root `just
   lint` recipe runs the same check in the frontend via `bunx biome check .`
   against the committed `biome.json`; do not commit code that Biome would
   reformat or flag.
2. **TypeScript** -- `tsc --noEmit` must report zero errors. `tsconfig.json`
   enables `strict`, `noUnusedLocals`, `noUnusedParameters`,
   `noFallthroughCasesInSwitch`, `isolatedModules`, and
   `verbatimModuleSyntax`, so the compiler is part of the review.
3. **Vitest** -- `bun run test` must pass.

Coverage is **reported, never gated**. `vite.config.ts` configures the v8
provider with `text` and `html` reporters (scoped to `src/pages/**`,
`src/components/**`, `src/hooks/**`, and `src/api/client.ts`); run
`bun run test:coverage` to see it. Do not add a coverage threshold that would
block a fork or a contributor on a percentage.

## 3. The generated API client

The OpenAPI document produced by the Rust backend is the single source of truth
for the API surface. The `typegen` recipe in the repo's `justfile` regenerates
`interface/src/api/schema.d.ts` from it: it exports the spec with
`cargo run --bin openapi_spec` and feeds it through
`bunx openapi-typescript` into `src/api/schema.d.ts`. Run `just typegen` to
regenerate, and `just check-typegen` to fail when the committed types are stale
relative to the spec.

- **Never hand-edit `schema.d.ts`.** It is generated, it is excluded from Biome
  (see the `!src/api/schema.d.ts` entry in `biome.json`), and it is committed to
  the repo whenever it changes.
- **Regenerate and commit together.** After any backend API or OpenAPI change,
  run `just typegen` and commit the regenerated `schema.d.ts` in the same change
  as the code that depends on it (the backend change, or the frontend code that
  uses the new types). Do not land schema changes in a commit separate from the
  code that relies on them, or `check-typegen` will catch the drift.
- **Reach the API only through the typed client** in `interface/src/api/client.ts`.
  That module creates a single `openapi-fetch` client typed against the
  generated schema:

  ```ts
  import createClient from "openapi-fetch";
  import type { paths } from "./schema";

  // baseUrl precedence: VITE_API_BASE_URL (build-time) -> Tauri sidecar
  // loopback -> "/" (same-origin browser default). See docs/api-endpoint.md.
  export const api = createClient<paths>({
    baseUrl: resolveApiBaseUrl(),
  });
  ```

  Import `api` from `../api/client`. Do **not** call raw `fetch` for backend
  requests -- raw `fetch` bypasses the path, method, and payload types that the
  generated client enforces.
- **Always destructure `{ data, error }` and throw on `error`.** Every call
  returns both, and the `openapi-fetch` client guarantees exactly one is
  defined: if `error` is truthy, throw it and do not touch `data`. The shipped
  pattern is:

  ```ts
  const { data, error } = await api.GET("/api/v1/todo");
  if (error) throw error;
  return data;
  ```

- **Keep the Tauri-aware `baseUrl`.** It is load-bearing (see section 11);
  do not hardcode an origin or remove the conditional.

## 4. Project structure

The frontend lives under `interface/src/` with a flat, predictable layout:

- `api/` -- the typed client (`client.ts`) and the generated `schema.d.ts`.
- `pages/` -- core pages, one component per file (`Home.tsx`), each with a
  co-located test.
- `plugins/` -- the plugin frontend layer: `contract.ts` + `registry.ts`
  (build-time discovery via `import.meta.glob`) and one dir per plugin
  (`plugins/todo/Todo.tsx`, `plugins/blog/Blog.tsx`) with co-located tests. The
  router builds its nav + routes from the discovered registry; the matching
  backend crate lives in the repo-root `plugins/<name>/`.
- `router.tsx` -- the TanStack Router tree: the root `Layout`, the routes, and
  the `Register` module augmentation.
- `main.tsx` -- the entry point that wires up the `QueryClient`, the providers,
  and `StrictMode`.
- `test-utils.tsx` -- shared test helpers (the `renderWithTheme` and
  `renderWithClient` wrappers).
- `styles.css` -- imports the Radix Themes stylesheet and the accent escape
  hatch (`theme/accent.css`), plus the body safe-area insets.

Keep to **one page per file**. New pages go in `pages/`; add a matching
`Name.test.tsx` alongside each new page so the co-located test convention holds.

Pages should import and compose shared UI rather than duplicate it. The reusable
layer lives in `src/components/` -- `ui/` primitives (`Button`, `Input`, `Badge`,
...) and `sections/` composites (`Toolbar`, `DataList`, `FilterBar`, ...), all
built on Radix Themes. Reach for those (or extend them) rather than copying
markup between pages. Test-only helpers (render wrappers and the like) belong in
`test-utils.tsx`, not in `pages/`.

## 5. Data fetching with TanStack Query

All server state flows through TanStack Query. Do not store fetched data in
`useState`; the query cache is the source of truth for server data.

**Query keys are arrays that include their inputs.** Use specific segments so
invalidation is precise: `["health"]`, `["items"]`, `["posts", filter]`,
`["posts", "stats"]`. When a query depends on an input (a filter, an id), put
that input in the key so the cache stays correct as it changes.

**Query functions are thin, inline, and throw on error.** A `queryFn` calls the
typed client, destructures `{ data, error }`, throws `error`, and returns
`data`:

```ts
const items = useQuery({
  queryKey: ["items"],
  queryFn: async () => {
    const { data, error } = await api.GET("/api/v1/todo");
    if (error) throw error;
    return data;
  },
});
```

Pass path params via `params: { path: { id } }` and query params via
`params: { query: { ... } }`. Build conditional query params explicitly. The
shipped pattern, for example, sends an empty object when no filter is active and
only includes `status` when one is selected:

```ts
params: { query: filter === "all" ? {} : { status: filter } },
```

rather than passing `undefined` blindly.

**Mutations use `useMutation` with an explicit `mutationFn` and an `onSuccess`
handler** that performs side effects and invalidation. Call `mutate(payload)`
with the payload as the argument; do not reach for ad hoc setters:

```ts
const queryClient = useQueryClient();

const createItem = useMutation({
  mutationFn: async (title: string) => {
    const { data, error } = await api.POST("/api/v1/todo", {
      body: { title },
    });
    if (error) throw error;
    return data;
  },
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ["items"] });
  },
});
```

**Invalidate with matching key segments.** On mutation success, call
`queryClient.invalidateQueries({ queryKey: [...] })` with the segments that
identify the affected queries, so dependent reads refresh. A shared
`invalidate` helper reused across a page's mutations (as in `Todo.tsx` and
`Blog.tsx`) keeps this consistent.

**Reflect mutation status in the UI.** The shipped pages gate submit controls on
`mutation.isPending` (for example `disabled={create.isPending}`). A `mutationFn`
that throws sets `mutation.isError`/`mutation.error`; if you need a user-facing
failure message, render those explicitly or add an `onError` handler. Do not
swallow mutation failures silently.

**Branch on `isLoading` and `isError` explicitly** in render logic. Render a
loading state and an error state as separate branches, then render the data.
Do not paper over possibly-undefined data with non-null assertions; use guarded
access (`items.data?.map(...)`, `items.data?.length === 0`) for the data branch:

```tsx
if (items.isLoading) return <p>Loading…</p>;
if (items.isError) return <p>Could not load items.</p>;
return <ul>{items.data?.map((item) => <li key={item.id}>{item.title}</li>)}</ul>;
```

## 6. Components and routing

**Components are functions with named exports.** Page components are declared
with the `function` keyword and exported by name (`HomePage`, `ItemsPage`,
`PostsPage`), not as arrow-function `const`s and not as default exports:

```tsx
export function ItemsPage() {
  // ...
}
```

**Routing uses TanStack Router** as set up in `router.tsx`:

- The root route is built with `createRootRoute` and renders a `Layout` that
  contains the nav and an `<Outlet />`.
- Page routes are built with `createRoute`, each passing `getParentRoute` and a
  `path`, with `component` set to the page function.
- The tree is composed with `rootRoute.addChildren([...])` and handed to
  `createRouter`.
- The router type is registered through a `declare module` `Register`
  augmentation so navigation is fully typed.

To add a page: write the page in `pages/`, create its route with `createRoute`,
add it to `rootRoute.addChildren([...])`, and add a nav `Link` in `Layout`.

**Navigation uses the Router `Link` component** with a `to` prop. TanStack Router
adds an `.active` class to the current route's link; the template surfaces it
through TanStack's `activeProps` (the active style plus `aria-current="page"`) in
`router.tsx`, not a CSS utility. Do not roll your own active-state tracking.

**Imports are relative with explicit depth** (`../api/client`, `./pages/Home`).
Do **not** introduce `@/` path aliases -- the repo does not use them and its
flat layout does not need them. If a future module chain grows deep enough that
relative paths become unwieldy, raise it as a team decision rather than adding an
alias unilaterally.

**State ownership:** use `useState` for local, ephemeral UI state (the current
filter, an input's draft value). Use the query cache for anything that comes
from or is written to the backend.

## 7. Types

- **Prefer types inferred from the generated client over hand-written
  interfaces** for API data. The shapes in `schema.d.ts` already describe
  requests and responses; let the typed `api` client supply them rather than
  re-declaring `interface Item { ... }` by hand, which would drift from the
  backend.
- **Use `as const` for literal unions** instead of writing the union out twice.
  The shipped pattern derives the type from the value:

  ```ts
  const FILTERS = ["all", "draft", "published", "archived"] as const;
  type Filter = (typeof FILTERS)[number];
  ```

- **Avoid non-null assertions (`!`).** Handle `null`/`undefined` with explicit
  branches or guarded access (`data?.length === 0`, the loading/error branches
  of section 5).
- **Avoid `any`.** With `strict` on, prefer precise types, `unknown` with
  narrowing, or types inferred from the client. If you truly cannot type
  something, narrow it as early as possible.

## 8. Formatting and lint rules

Biome owns formatting and linting for JavaScript and TypeScript; its config is
`interface/biome.json`. Do not hand-format against it -- run `bun run format`
and let Biome decide.

The enforced formatter settings are:

- 2-space indentation (`indentStyle: "space"`, `indentWidth: 2`).
- 80-column line width (`lineWidth: 80`).
- Double quotes (`quoteStyle: "double"`).
- Semicolons always (`semicolons: "always"`).
- Trailing commas everywhere (`trailingCommas: "all"`).

The linter is enabled with Biome's default rule set; the assist actions are
turned off (`assist.enabled: false`). One default rule is worth calling out
because it shapes everyday markup:

- `useButtonType` -- every `<button>` needs an explicit `type` (see section 9).

For the full list of active rules and their rationale, see the Biome linter
documentation at <https://biomejs.dev/linter/>.

Excluded from Biome (via the `files.includes` negations in `biome.json`):
`src/api/schema.d.ts` (generated) and all `*.css` files (the Radix Themes
stylesheet plus the generated accent scale in `theme/accent.css`), plus
`node_modules`, `dist`, and `coverage`. Do not work around these exclusions by
reformatting generated or CSS files by hand.

## 9. Styling and accessibility

**Styling is Radix Themes (`@radix-ui/themes`).** Components are styled with
Themes components and their props (`size`, `color`, `variant`, `radius`,
`weight`) and laid out with the Themes primitives (`Flex`, `Box`, `Grid`,
`Container`, `Section`) -- not utility classes. The full vocabulary (every prop
and its allowed values) lives in `docs/radix-reference.md`.

**Re-theme globally, not per component.** `interface/src/theme/theme.config.ts`
is the single source of truth for the accent, gray, panel background, radius, and
scaling; the whole app restyles from there (or via the `configure-theme` skill).
A custom brand hue that no built-in scale matches goes in `theme/accent.css`.
Light/dark is driven by the `.dark` class on `<html>` (`ThemeProvider` plus the
pre-hydration script in `index.html`), which Themes reads natively -- do not set
`appearance` on `<Theme>`.

**Colors come from the Radix scales,** addressed through the `color` prop or the
CSS variables Themes exposes (`var(--accent-9)`, `var(--gray-11)`,
`var(--gray-a6)`). Reach for a Themes prop first; drop to an inline `style` with a
Radix variable only for the rare gap (the active nav link in `router.tsx` is the
canonical example). Per-instance spacing uses the layout props (`m`, `mt`, `px`,
`gap`), not bespoke CSS.

**`styles.css` is minimal:** it imports the Themes stylesheet and `accent.css`,
then keeps only the body safe-area insets for mobile / Tauri. It is excluded from
Biome; do not lint or hand-format it, and prefer Themes props over growing
bespoke CSS.

Accessibility expectations:

- **Buttons that do not submit a form set `type="button"`** -- enforced by
  `useButtonType`. Action buttons (Add, Delete, Publish, Archive, filter
  toggles) all carry `type="button"` in the shipped pages.
- Prefer **semantic, keyboard-reachable elements**: real `<button>` and `<a>`
  (via the Router `Link`) for interactive controls, headings for structure,
  lists for collections. Do not attach click handlers to non-interactive
  elements like `<div>`.

## 10. Testing

Tests use **Vitest** plus **Testing Library**, configured in `vite.config.ts`
with the `jsdom` environment and `globals: true`.

- **Co-locate tests** beside the page as `Name.test.tsx` (`Items.test.tsx` next
  to `Todo.tsx`).
- **Mock the typed client, not the network.** Define the mock functions with
  `vi.hoisted` so they exist before the module factory runs, then
  `vi.mock("../api/client", ...)` to substitute the `api` object. This is the
  exact pattern in `Items.test.tsx`:

  ```ts
  const { getMock } = vi.hoisted(() => ({ getMock: vi.fn() }));

  vi.mock(
    "../api/client",
    () =>
      ({
        api: { GET: getMock, POST: vi.fn(), DELETE: vi.fn() },
      }) as unknown as typeof import("../api/client"),
  );
  ```

  Have each mocked method resolve to the `{ data, error }` shape the real client
  returns (for a success, `{ data: ..., error: undefined }`).
- **Render with `renderWithClient`** from `test-utils.tsx`. It wraps the
  component in a fresh `QueryClientProvider` whose queries have `retry: false`,
  so a failed query surfaces immediately instead of retrying through the test.
- **Assert on user-visible output.** Use `screen.getByText(...)` for what the
  user sees, and wrap async expectations in `waitFor(...)`. Do not assert on
  query internals or component implementation details.

## 11. Desktop / Tauri note

The optional Tauri 2 desktop shell runs the backend binary as a sidecar. Inside
Tauri the page origin is `tauri://localhost`, so relative API paths would not
reach the server. The client in `api/client.ts` detects Tauri via
`"__TAURI_INTERNALS__" in window` and switches `baseUrl` to the sidecar origin
`http://127.0.0.1:8080`; in the browser and the Vite dev server (which proxies
`/api`) it uses `/`. A build-time `VITE_API_BASE_URL` overrides both when set (see `docs/api-endpoint.md`). **Do not remove or hardcode around this logic** -- it is
what makes the same frontend work in both the browser and the desktop shell.
