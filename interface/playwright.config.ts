import { defineConfig } from "@playwright/test";

// Accessibility smoke: load each page in a real browser and run axe-core, which
// catches contrast, focus, and layout issues that jsdom + jest-axe cannot.
// Run with `just a11y` after a one-time `bunx playwright install chromium`.
export default defineConfig({
  testDir: "./e2e",
  // Keep run artifacts out of the Biome-linted / git-tracked tree.
  outputDir: "node_modules/.playwright",
  reporter: "list",
  webServer: {
    command: "bun run dev",
    url: "http://localhost:5173",
    reuseExistingServer: true,
    timeout: 60_000,
  },
  use: { baseURL: "http://localhost:5173" },
});
