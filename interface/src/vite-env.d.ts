/// <reference types="vite/client" />

interface ImportMetaEnv {
  /**
   * Absolute base URL for the JSON API (e.g. `http://127.0.0.1:8080`), baked
   * into the build so a browser or desktop build can retarget the API without
   * code edits. Falls back, in `src/api/client.ts`, to `/` in the browser and
   * the sidecar loopback URL in the Tauri desktop shell. See
   * `docs/api-endpoint.md` for the full cross-surface contract.
   */
  readonly VITE_API_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
