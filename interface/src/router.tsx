import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
} from "@tanstack/react-router";
import { CategoriesPage } from "./pages/Categories";
import { DashboardPage } from "./pages/Dashboard";
import { ExpensesPage } from "./pages/Expenses";
import { SettingsPage } from "./pages/Settings";

const navLink = "text-sm text-zinc-400 [&.active]:text-zinc-100";

function Layout() {
  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      <nav className="mb-8 flex items-center gap-6 border-b border-zinc-800 pb-4">
        <span className="font-semibold tracking-tight">Expense Tracker</span>
        <Link to="/" className={navLink}>
          Dashboard
        </Link>
        <Link to="/expenses" className={navLink}>
          Expenses
        </Link>
        <Link to="/categories" className={navLink}>
          Categories
        </Link>
        <Link to="/settings" className={navLink}>
          Settings
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
  component: DashboardPage,
});

const expensesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/expenses",
  component: ExpensesPage,
});

const categoriesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/categories",
  component: CategoriesPage,
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: SettingsPage,
});

export const router = createRouter({
  routeTree: rootRoute.addChildren([
    indexRoute,
    expensesRoute,
    categoriesRoute,
    settingsRoute,
  ]),
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
