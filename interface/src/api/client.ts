import createClient from "openapi-fetch";
import type { paths } from "./schema";

// Inside the Tauri desktop shell the page origin is tauri://localhost,
// so API calls must target the sidecar's absolute URL. In the browser
// (and Vite dev server, which proxies /api) relative paths work.
const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const api = createClient<paths>({
  baseUrl: isTauri ? "http://127.0.0.1:8080" : "/",
});
