import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    // Dev convenience: the Vite dev server proxies API calls to the
    // Rust backend, so the frontend can always fetch relative paths.
    proxy: { "/api": "http://localhost:8080" },
  },
});
