import type { PluginRoute } from "../contract";

// The frontend descriptor for the `blog` plugin (backend crate in plugins/blog/).
export default {
  path: "/blog",
  label: "Blog",
  component: () => import("./Blog").then((module) => module.BlogPage),
} satisfies PluginRoute;
