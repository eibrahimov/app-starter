# API endpoint contract

Every surface — the browser SPA, the Tauri desktop shell, and any future CLI
(#C14) — has to agree on **one thing**: where the JSON API lives. This file is
that contract. There is **no behavior change at the defaults**; the point is to
make the coupling explicit and overridable instead of three independent
hardcodes that silently drift apart when you change the port.

## The knobs

| Surface | Knob | Kind | Default | Set in |
|---|---|---|---|---|
| Server / CLI | `PORT` | runtime env / `--port` | `8080` | `.env`, shell, or flag |
| Server / CLI | `DATABASE_URL` | runtime env / `--database-url` | `sqlite://data/app.db?mode=rwc` | `.env`, shell, or flag |
| Browser SPA | `VITE_API_BASE_URL` | **build-time** env | `/` (same-origin; relative) | `interface/.env` or build env |
| Desktop webview | `VITE_API_BASE_URL` | **build-time** env | `http://127.0.0.1:8080` (sidecar loopback) | build env |
| Desktop sidecar | `PORT` | runtime env (read by the shell, forwarded to the sidecar) | `8080` | desktop launch env |

The server already reads `PORT`/`DATABASE_URL` via clap (`src/main.rs`), so the
CLI surface is covered today and the same variables carry forward to #C14.

## How each surface resolves the base URL

- **Server (`src/main.rs`).** Binds `0.0.0.0:$PORT`. Authoritative for where the
  API actually listens.
- **Browser SPA (`interface/src/api/client.ts`).** Uses
  `import.meta.env.VITE_API_BASE_URL`, falling back to `/` — a same-origin
  relative base, so the one binary that serves both the API and the built UI
  needs no configuration. In `bun run dev` the Vite dev server proxies `/api` to
  `http://localhost:$PORT` (`vite.config.ts` reads `PORT`), so changing the
  backend port does not silently break web dev.
- **Desktop (`interface/src/api/client.ts` + `desktop/src-tauri/src/main.rs`).**
  The webview origin is `tauri://localhost` (or the dev-server URL), so relative
  paths cannot reach the backend; the client uses `VITE_API_BASE_URL`, falling
  back to the sidecar's loopback `http://127.0.0.1:8080`. The shell spawns the
  sidecar with an explicit `PORT` (read from its own environment, default
  `8080`) rather than relying on the server binary's default. **These two are the
  two ends of one contract: the webview's `VITE_API_BASE_URL` and the sidecar's
  `PORT` must agree.** The Tauri-aware branch in `client.ts` is a hard rule
  (`AGENTS.md`) — extend it, never remove it.

## Retargeting recipes

- **Run the server on a different port (web/Docker):**

  ```sh
  PORT=9090 cargo run           # or set PORT in .env
  ```

  For `bun run dev`, set the same `PORT` in the calling shell so the `/api` proxy
  follows (it is read once at Vite startup, not per request):

  ```sh
  PORT=9090 just frontend-dev
  ```

- **Point a browser build at a remote API:**

  ```sh
  # interface/.env  (copy from interface/.env.example)
  VITE_API_BASE_URL=https://api.example.com
  ```

  Two browser constraints apply when the SPA and API are different origins:
  - **CORS is required.** `CorsLayer::permissive()` (`src/api.rs`) currently allows
    it; if you tighten CORS before public exposure (see
    `docs/production-readiness.md`), the policy must still allow the SPA's origin.
  - **No mixed content.** An `https`-served SPA cannot call an `http` API, so
    `VITE_API_BASE_URL` must be `https` in that case.

- **Run the desktop shell on a different port:** set `PORT` for the desktop
  launch **and** build the frontend with a matching `VITE_API_BASE_URL`:

  ```sh
  PORT=9090 VITE_API_BASE_URL=http://127.0.0.1:9090 bun run build   # in interface/
  PORT=9090 bun run dev                                             # in desktop/
  ```

## Notes

- `VITE_API_BASE_URL` is **baked in at build time** (it is `import.meta.env.*`),
  not read at runtime — a packaged build is fixed to the value present when its
  frontend was built. `PORT`/`DATABASE_URL` are runtime values.
- Mobile is out of scope: the sidecar model does not run on iOS/Android, and OS
  WebView cleartext policy blocks `http://localhost` regardless. See
  `docs/desktop-features.md` §6.
