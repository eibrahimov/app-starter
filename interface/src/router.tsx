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

const navLinkClass =
  "inline-flex items-center text-sm text-muted-foreground coarse:min-h-11 [&.active]:text-foreground";

function Layout() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-6 sm:py-8">
      <nav className="mb-8 flex flex-wrap items-center gap-4 border-b border-border pb-4 sm:gap-6">
        <span className="font-semibold tracking-tight">App Starter</span>
        <Link to="/" className={navLinkClass}>
          Home
        </Link>
        <Link to="/items" className={navLinkClass}>
          Items
        </Link>
        <Link to="/posts" className={navLinkClass}>
          Posts
        </Link>
        <span className="ml-auto">
          <ThemeToggle />
        </span>
      </nav>
      <Outlet />
    </div>
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
