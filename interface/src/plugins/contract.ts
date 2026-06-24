import type { ComponentType } from "react";

/**
 * The descriptor a plugin's `frontend/plugin.tsx` default-exports. The SPA
 * discovers these at build time (see `registry.ts`) and builds its route tree +
 * nav by iterating them. The page component is loaded lazily so each plugin
 * code-splits into its own chunk.
 *
 * Navigation is NOT statically typed (a runtime-mapped route list erases the
 * path literals TanStack would need); a guard test instead asserts every path is
 * unique and well-formed (docs/plugin-framework.md §3 [B4]).
 */
export interface PluginRoute {
  /** Route path, e.g. "/todo". Must be unique across plugins and start with "/". */
  path: string;
  /** Nav label, e.g. "Todo". */
  label: string;
  /** Lazy loader for the page component (its own code-split chunk). */
  component: () => Promise<ComponentType>;
}
