import type { PluginRoute } from "./contract";

// Discover every plugin's frontend descriptor at build time. The glob is
// EAGER -- the descriptors are tiny (path/label + a lazy component loader), so
// loading them up front lets the router build its tree and nav synchronously.
// The actual page each descriptor points at stays lazy (its own chunk), so
// bundle size is unaffected. Plugin frontends live under interface/src/plugins/
// (one dir per plugin) so they resolve the shared UI deps + typed client the
// normal way; the backend crate lives separately in plugins/<name>/.
const modules = import.meta.glob<{ default: PluginRoute }>("./*/plugin.tsx", {
  eager: true,
});

/** Every discovered plugin route, ordered by path for a stable nav. */
export const pluginRoutes: PluginRoute[] = Object.values(modules)
  .map((module) => module.default)
  .sort((a, b) => a.path.localeCompare(b.path));
