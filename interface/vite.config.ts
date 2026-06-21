import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// This config runs in Node; declare just the slice of `process` we read so we
// avoid depending on @types/node globals (which clash with DOM timer types in
// src/). The proxy target follows PORT -- the server's own env var -- so a
// non-default backend port does not silently break web dev (see
// docs/api-endpoint.md).
declare const process: { env: Record<string, string | undefined> };
const backendPort = process.env.PORT ?? "8080";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    // Dev convenience: the Vite dev server proxies API calls to the Rust
    // backend, so the frontend can always fetch relative paths.
    proxy: { "/api": `http://localhost:${backendPort}` },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
    // Scope to src so Vitest never tries to run the Playwright e2e/*.spec.ts.
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      // Coverage is reported, never enforced as a gate -- forks should not be
      // blocked on a percentage. Run `bun run test:coverage` to see it.
      provider: "v8",
      reporter: ["text", "html"],
      include: [
        "src/pages/**",
        "src/components/**",
        "src/hooks/**",
        "src/api/client.ts",
      ],
    },
  },
});
