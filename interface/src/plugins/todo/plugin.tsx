import type { PluginRoute } from "../contract";

// The frontend descriptor for the `todo` plugin. The SPA discovers this via
// `interface/src/plugins/registry.ts`. The page is loaded lazily so it ships in
// its own chunk. (The plugin's backend crate lives in `plugins/todo/`; its
// frontend lives here under interface/src so it resolves the shared UI deps and
// the typed client the normal way -- see docs/plugin-framework-impl-status.md.)
export default {
  path: "/todo",
  label: "Todo",
  component: () => import("./Todo").then((module) => module.TodoPage),
} satisfies PluginRoute;
