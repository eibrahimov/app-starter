import { describe, expect, it } from "vitest";
import { pluginRoutes } from "./registry";

// Navigation runtime guard (docs/plugin-framework.md §3 [B4]). Because plugin
// nav is runtime-mapped (not statically typed), this asserts the discovered
// route list is coherent: unique, well-formed paths with labels and a loader.
describe("plugin route registry", () => {
  it("has unique, well-formed paths with a label and a component loader", () => {
    const paths = pluginRoutes.map((route) => route.path);
    expect(new Set(paths).size).toBe(paths.length);

    for (const route of pluginRoutes) {
      expect(route.path.startsWith("/")).toBe(true);
      expect(route.label.length).toBeGreaterThan(0);
      expect(typeof route.component).toBe("function");
    }
  });

  it("discovers the todo plugin", () => {
    expect(pluginRoutes.some((route) => route.path === "/todo")).toBe(true);
  });
});
