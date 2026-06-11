import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
} from "@tanstack/react-router";
import { HomePage } from "./pages/Home";
import { ItemsPage } from "./pages/Items";

function Layout() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      <nav className="mb-8 flex items-center gap-6 border-b border-zinc-800 pb-4">
        <span className="font-semibold tracking-tight">App Starter</span>
        <Link to="/" className="text-sm text-zinc-400 [&.active]:text-zinc-100">
          Home
        </Link>
        <Link
          to="/items"
          className="text-sm text-zinc-400 [&.active]:text-zinc-100"
        >
          Items
        </Link>
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

export const router = createRouter({
  routeTree: rootRoute.addChildren([indexRoute, itemsRoute]),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
