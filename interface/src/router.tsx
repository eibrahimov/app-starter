import {
  Box,
  Container,
  Flex,
  Heading,
  Link as ThemesLink,
} from "@radix-ui/themes";
import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useRouterState,
} from "@tanstack/react-router";
import { type ComponentType, lazy, Suspense } from "react";
import { ErrorBoundary } from "react-error-boundary";
import { ErrorFallback } from "./components/app/ErrorFallback";
import { reportBoundaryError } from "./components/app/reportBoundaryError";
import { ThemeToggle } from "./components/theme/ThemeToggle";
import { HomePage } from "./pages/Home";
import { PostsPage } from "./pages/Posts";
import { pluginRoutes } from "./plugins/registry";

// Core nav entries: Home plus the still-core resources. Plugin routes (e.g.
// /todo) are appended from the discovered registry below, so adding a plugin
// never edits this file.
const CORE_NAV = [
  { to: "/", label: "Home" },
  { to: "/posts", label: "Posts" },
] as const;

// TanStack Router adds an `.active` class to the <Link> for the current route.
// We render each nav item as a Themes `Link` via `asChild` so it inherits the
// accessible Themes link styling and touch sizing. The active page is surfaced
// with the high-contrast accent treatment (full-strength accent text via
// `--accent-12` plus a weight bump) applied through TanStack's `activeProps`
// inline style. Inactive links use the low-contrast gray (`--gray-11`).
const ACTIVE_STYLE = {
  color: "var(--accent-12)",
  fontWeight: 600,
  textDecoration: "none",
} as const;

const INACTIVE_STYLE = {
  color: "var(--gray-11)",
  textDecoration: "none",
} as const;

function NavLink({ to, children }: { to: string; children: string }) {
  return (
    <ThemesLink asChild size="2">
      <Link
        to={to}
        style={INACTIVE_STYLE}
        activeProps={{ style: ACTIVE_STYLE, "aria-current": "page" }}
      >
        {children}
      </Link>
    </ThemesLink>
  );
}

function Layout() {
  // Reset the route-level boundary on navigation so a crash on one page does
  // not stick after the user moves to another route.
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  return (
    <Container size="2" px="4" py={{ initial: "6", sm: "8" }}>
      <Box
        mb="8"
        pb="4"
        style={{ borderBottom: "1px solid var(--gray-a6)" }}
        asChild
      >
        <nav>
          <Flex align="center" wrap="wrap" gap={{ initial: "4", sm: "6" }}>
            <Heading as="h1" size="3" weight="bold" trim="both">
              App Starter
            </Heading>
            {CORE_NAV.map((entry) => (
              <NavLink key={entry.to} to={entry.to}>
                {entry.label}
              </NavLink>
            ))}
            {pluginRoutes.map((route) => (
              <NavLink key={route.path} to={route.path}>
                {route.label}
              </NavLink>
            ))}
            <Box flexGrow="1" />
            <ThemeToggle />
          </Flex>
        </nav>
      </Box>
      {/* Route-level boundary: a render error in a page falls back inline while
          the nav shell above stays usable, instead of white-screening the SPA. */}
      <ErrorBoundary
        FallbackComponent={ErrorFallback}
        onError={reportBoundaryError}
        resetKeys={[pathname]}
      >
        <Outlet />
      </ErrorBoundary>
    </Container>
  );
}

const rootRoute = createRootRoute({ component: Layout });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});

const postsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/posts",
  component: PostsPage,
});

// Wrap a plugin's lazy component loader in React.lazy so the page ships in its
// own chunk and only loads when its route is visited.
function lazyPluginPage(load: () => Promise<ComponentType>) {
  const Lazy = lazy(() => load().then((component) => ({ default: component })));
  return function PluginPage() {
    return (
      <Suspense fallback={null}>
        <Lazy />
      </Suspense>
    );
  };
}

// Build a route per discovered plugin. Navigation is not statically typed (the
// path literals are erased by the runtime mapping); the registry guard test
// asserts the paths are unique and well-formed (docs/plugin-framework.md §3).
const pluginRouteObjects = pluginRoutes.map((route) =>
  createRoute({
    getParentRoute: () => rootRoute,
    path: route.path,
    component: lazyPluginPage(route.component),
  }),
);

export const router = createRouter({
  routeTree: rootRoute.addChildren([
    indexRoute,
    postsRoute,
    ...pluginRouteObjects,
  ]),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
