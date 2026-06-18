import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    // Dev convenience: the Vite dev server proxies API calls to the
    // Rust backend, so the frontend can always fetch relative paths.
    proxy: { "/api": "http://localhost:8080" },
  },
  test: {
    environment: "jsdom",
    globals: true,
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
