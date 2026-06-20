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
} from "@tanstack/react-router";
import { ThemeToggle } from "./components/theme/ThemeToggle";
import { HomePage } from "./pages/Home";
import { ItemsPage } from "./pages/Items";
import { PostsPage } from "./pages/Posts";

// TanStack Router adds an `.active` class to the <Link> for the current route.
// We render each nav item as a Themes `Link` via `asChild` so it inherits the
// accessible Themes link styling and touch sizing. The active page is surfaced
// with the high-contrast accent treatment (full-strength accent text via
// `--accent-12` plus a weight bump) applied through TanStack's `activeProps`
// inline style, so the current route reads as active without any Tailwind
// utility. Inactive links use the low-contrast gray (`--gray-11`).
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
            <NavLink to="/">Home</NavLink>
            <NavLink to="/items">Items</NavLink>
            <NavLink to="/posts">Posts</NavLink>
            <Box flexGrow="1" />
            <ThemeToggle />
          </Flex>
        </nav>
      </Box>
      <Outlet />
    </Container>
  );
}

const rootRoute = createRootRoute({ component: Layout });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});

const itemsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/items",
  component: ItemsPage,
});

const postsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/posts",
  component: PostsPage,
});

export const router = createRouter({
  routeTree: rootRoute.addChildren([indexRoute, itemsRoute, postsRoute]),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
