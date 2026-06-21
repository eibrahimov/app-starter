import createClient from "openapi-fetch";
import type { paths } from "./schema";

// One overridable contract for where the API lives (see docs/api-endpoint.md).
// VITE_API_BASE_URL is baked in at build time and lets a browser or desktop
// build retarget the API without code edits; it falls back to the historical
// per-mode defaults, so builds that set nothing are unchanged.
export function resolveApiBaseUrl(): string {
  // Inside the Tauri desktop shell the page origin is tauri://localhost (or the
  // dev-server URL), so API calls must target the sidecar's absolute loopback
  // URL. In the browser (and the Vite dev server, which proxies /api) relative
  // paths work. This Tauri-aware branch is a hard rule (AGENTS.md) -- extend it,
  // never remove it.
  const isTauri =
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  // Treat an unset OR blank value as "use the default": `??` alone would let an
  // explicitly-empty VITE_API_BASE_URL="" through as the base URL.
  const override = import.meta.env.VITE_API_BASE_URL?.trim();
  if (override) {
    return override;
  }
  return isTauri ? "http://127.0.0.1:8080" : "/";
}

export const api = createClient<paths>({ baseUrl: resolveApiBaseUrl() });
