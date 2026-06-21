import { describe, expect, it } from "vitest";
import { toneColor } from "../components/ui/Badge";
import { VARIANT_MAP } from "../components/ui/Button";
import catalogRaw from "./radix.catalog.json?raw";
import * as icons from "./icons";
import { themeConfig } from "./theme.config";

// Drift guard for radix.catalog.json -- the machine-readable companion to
// docs/radix-reference.md. The catalog is hand-maintained prose-as-data, so this
// test ties its two source-of-truth-bearing sections back to the real code:
//   - the icon allow-list must equal icons.ts exactly, and
//   - every value set in theme.config.ts must be an enumerated allowed value.
// If either drifts, this fails -- so the catalog cannot silently lie to an agent.
// (Imported with Vite's `?raw` -- a plain string typed by vite/client -- because
// tsconfig has no resolveJsonModule; parse it here.)
const catalog = JSON.parse(catalogRaw);

describe("radix.catalog.json", () => {
  it("icon allow-list matches icons.ts exactly", () => {
    const exported = Object.keys(icons).sort();
    const cataloged = [...catalog.icons.allow].sort();
    expect(cataloged).toEqual(exported);
  });

  it("enumerates every prop set in theme.config.ts with the current value allowed", () => {
    for (const [prop, value] of Object.entries(themeConfig)) {
      const spec = catalog.theme.props[prop];
      // The configured prop must exist in the catalog...
      expect(
        spec,
        `theme.config.ts sets "${prop}" but the catalog omits it`,
      ).toBeDefined();
      // ...and its current value must be one the catalog lists as allowed.
      expect(
        spec.values,
        `catalog lists "${prop}"=${JSON.stringify(value)} as not allowed`,
      ).toContain(String(value));
    }
  });

  it("does not set appearance (driven by the .dark class, not config)", () => {
    // Mirrors the invariant in theme.config.ts and configure-theme: passing
    // `appearance` reintroduces a load flash. The catalog flags it doNotSet.
    expect(themeConfig).not.toHaveProperty("appearance");
    expect(catalog.theme.props.appearance.doNotSet).toBe(true);
  });

  it("Badge/Button wrapper value sets match the catalog", () => {
    // The wrappers are the source of truth for the template's public tone and
    // variant names; pin the catalog's copies to them so editing either map
    // (e.g. adding a Badge tone) without updating the catalog fails here.
    expect([...catalog.templatePrimitives.Badge.values].sort()).toEqual(
      Object.keys(toneColor).sort(),
    );
    expect([...catalog.templatePrimitives.Button.values].sort()).toEqual(
      Object.keys(VARIANT_MAP).sort(),
    );
  });
});
