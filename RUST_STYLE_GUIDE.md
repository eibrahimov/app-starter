# Rust Style Guide

Conventions for the App Starter backend. Follow these exactly. When in doubt,
consistency with existing code wins over personal preference. Every descriptive
rule here is a pattern already shipped in `src/`; a few clearly-marked
recommendations are flagged "adopt going forward."

App Starter is an AI-native foundation for building intelligent ("Software 3.0")
applications -- by humans and AI coding agents together (see [VISION.md](VISION.md)).
Its current backend spine is axum + sqlx + utoipa, and the AI/agent layer is a
first-class growth axis, not an afterthought: see **Building AI and Agent
Features** for how that capability is meant to grow on top of the typed contract.

For the end-to-end recipe that ties a resource together across migration,
queries, handlers, OpenAPI, and generated types, see [AGENTS.md](AGENTS.md). For
the frontend that consumes the generated types, see
[TS_STYLE_GUIDE.md](TS_STYLE_GUIDE.md). When this guide and the code disagree,
treat it as a bug in one of them and fix it.

## Project Structure

A single binary plus library crate. The binary is thin; all domain and HTTP
logic lives in the library so integration tests and binaries can both reach it.

```text
src/
├── main.rs          — process entry: env, logging, args, serve, shutdown
├── lib.rs           — crate root: pub mod declarations + AppState
├── error.rs         — AppError and its IntoResponse mapping
├── db.rs            — pool init, migrations, URL redaction
├── frontend.rs      — embedded SPA serving
├── items.rs         — example domain module: types + queries
├── posts.rs         — second example: status lifecycle, filters, stats
├── api.rs           — router assembly + the OpenAPI document
├── api/             — per-resource HTTP modules (never api/mod.rs)
│   ├── health.rs    — readiness probe handler
│   ├── items.rs     — items HTTP handlers
│   └── posts.rs     — posts HTTP handlers
└── bin/
    └── openapi_spec.rs  — prints the spec for typegen
```

Each resource is split across two files: a **domain module** at
`src/<resource>.rs` that knows sqlx and the database, and an **HTTP module** at
`src/api/<resource>.rs` that knows axum and `AppError`. Keep the layers from
leaking: handlers never contain SQL; domain functions never return HTTP types.

`src/lib.rs` is the crate root. It declares public modules and defines the shared
state, which carries only the pool:

```rust
pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod items;
pub mod posts;
pub mod seed;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
}
```

**No `mod.rs` files.** Use the `<module>.rs` + `<module>/` layout, never
`<module>/mod.rs`: a uniquely named module file is far easier for humans and AI
agents to locate and grep than one of many identical `mod.rs` files. `src/api.rs`
is the api module root (it holds the router and the OpenAPI document); the
per-resource handler modules live alongside it in `src/api/`. CI fails if any
`mod.rs` is committed.

## Toolchain and Lints

- **Edition 2024**, MSRV pinned to `1.88` in `Cargo.toml` (`rust-version`).
  Edition 2024 needs Rust `>= 1.85`, but the let-chains in `src/db.rs` need
  `>= 1.88`, so the pin makes `cargo` emit a clear "requires Rust >= X" error.
- **`rust-toolchain.toml`** pins the toolchain to `1.96.0` — newer than the `1.88`
  MSRV above; it is the exact version CI builds with — plus the `rustfmt` and
  `clippy` components. All are mandatory; do not work around them.
- **Format with default `rustfmt`.** There is no `rustfmt.toml`. Run
  `cargo fmt --all`; CI runs `cargo fmt --all -- --check` and fails on any diff.
- **Clippy is a hard gate.** Warnings are errors:

  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

  Fix the lint rather than suppressing it. Reach for `#[allow(...)]` only with a
  written reason and the narrowest scope.
- **Supply chain** is gated by `cargo-deny` from `deny.toml` (see Dependencies
  and Licensing).
- **Local commands.** `just lint` runs the fast gate (fmt check, clippy,
  frontend Biome, `tsc`). `just test` runs backend tests. `just verify` runs the
  full CI set. After any API or OpenAPI change also run `just typegen` and
  `just check-typegen`.

**Recommended, not yet wired (adopt going forward).** A `[lints.clippy]` block
that forbids debugging and placeholder macros keeps them out of shipped code:

```toml
[lints.clippy]
dbg_macro = "forbid"
todo = "forbid"
unimplemented = "forbid"
```

This is not in `Cargo.toml` today. Until it is, use `tracing::debug!` for debug
output and `// TODO:` comments for tracked work rather than `dbg!`/`todo!()`.

## Imports

Imports are managed by `rustfmt`; do not hand-reorder them. Group items that
share a path prefix into the nested brace form rather than repeating the prefix:

```rust
use crate::AppState;
use crate::error::AppError;
use crate::items::{self, CreateItem, Item};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
```

Prefer importing the items you use over fully qualifying at every call site,
except where a fully qualified path reads more clearly (`tracing::error!`,
`sqlx::query_as`).

## Naming

Standard Rust naming, which clippy and the API guidelines enforce. Avoid
abbreviations: spell out `database_url`, `request_id`, `status`.

| Kind | Convention | Examples |
|------|-----------|----------|
| Functions, methods, variables, modules | `snake_case`, verb-first for actions | `list`, `create_item`, `redact_url`, `database_url` |
| Types, traits, enum variants | `PascalCase`, descriptive | `AppError`, `PostStatus`, `CreatePost`, `AppState` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_BODY_BYTES`, `REQUEST_TIMEOUT`, `SELECT_COLUMNS` |
| Boolean accessors | read as a question | `is_empty`, `is_ok` |
| Domain query functions | the operation, not the sqlx call | `list`, `get`, `create`, `toggle`, `remove`, `publish` |

## Visibility

Keep the public surface small; expose only what callers actually need.

- **Fields and items are private by default.** `AppState` exposes `pub pool`
  because every handler reads it; most fields do not need to be public.
- **`pub mod` in `src/lib.rs` is the library's public surface.** Declaring a
  module `pub mod` makes it reachable by the integration tests (`tests/api.rs`)
  and the binaries (`src/bin/openapi_spec.rs`). Declare a module there only when
  it is meant to be reached that way; keep private helpers as non-`pub` items.
- **Handlers and domain functions are `pub`** so the router and tests can call
  them; module-internal helpers stay private to their file.
- **Recommended, adopt going forward:** as modules grow, prefer `pub(crate)` for
  items several modules share internally but that are not part of the public API,
  and gate test-only accessors behind `#[cfg(test)]`. The current code is small
  enough not to need this yet; reach for it before widening a `pub`.

## Types and Derives

Model data with concrete structs and enums and derive only what each type needs.

**Row and response structs** (read from the database, returned to clients) derive
`Debug`, `Serialize`, `sqlx::FromRow`, and `utoipa::ToSchema`:

```rust
#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}
```

**Request payloads** (deserialized from the client) derive `Debug`,
`Deserialize`, and `ToSchema`. Use `#[serde(default)]` for optional fields:

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePost {
    pub title: String,
    #[serde(default)]
    pub body: String,
}
```

**Query-parameter structs** add `utoipa::IntoParams`; **aggregates with a natural
zero** add `Default` and use `..Default::default()` for partial init:

```rust
#[derive(Debug, Default, Serialize, ToSchema)]
pub struct PostStats {
    pub draft: i64,
    pub published: i64,
    pub archived: i64,
}
// let mut stats = PostStats::default();
```

Field conventions:

- **Timestamps** are `chrono::DateTime<Utc>`, captured with `Utc::now()`, named
  with an `_at` suffix (`created_at`, `published_at`). A required timestamp is
  non-null; an optional one is `Option<DateTime<Utc>>`.
- **Identifiers** are generated application-side with `Uuid::new_v4().to_string()`
  and stored as `String`. The database column is a TEXT primary key.
- **Make invalid states unrepresentable:** `Option<T>` for genuinely optional
  fields, enums for fixed choices, and the narrowest type a function needs.
- **Recommended, adopt going forward:** add `#[non_exhaustive]` to public
  response structs (`Item`, `Post`, `Health`, `PostStats`) so adding a field is
  not a breaking change for downstream Rust consumers. The structs are plain
  today; adopt this when contract stability outweighs struct-literal convenience.

## Enums and Validated State

Model closed sets of values as enums, not bare strings. `PostStatus` is the
worked example. Deriving `serde` + `utoipa::ToSchema` + `sqlx::Type` (all with a
matching lowercase rename) expresses the vocabulary once: the same strings are
the stored TEXT value, the wire JSON, and the OpenAPI/TypeScript enum. It still
carries `as_str` for binding into queries and a `parse` that returns `None` for
unknown input so the list handler can reject `?status=bogus` as a clean 400
rather than a generic deserialization rejection.

```rust
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl PostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
            PostStatus::Archived => "archived",
        }
    }

    /// Returns `None` for unknown values so handlers can reject them as 400.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(PostStatus::Draft),
            "published" => Some(PostStatus::Published),
            "archived" => Some(PostStatus::Archived),
            _ => None,
        }
    }
}
```

`Post.status` is typed as `PostStatus`, and `sqlx` round-trips it to/from the
TEXT column, so the contract describes the closed set the backend already
enforces (the generated TypeScript narrows `status` from `string` to
`"draft" | "published" | "archived"`). Because the wire strings are unchanged,
this is an additive refinement within `/api/v1`. **State transitions** are
enforced where the data lives: the SQL update is conditional on the current
state, and the handler checks the precondition first (see HTTP Handlers).

## Error Handling

`AppError` in `src/error.rs` owns the mapping from internal errors to HTTP
responses. Handlers never build error responses themselves.

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

The `IntoResponse` impl matches on `&self`, logs internal failures, and returns a
`{"error": message}` body. Internal detail is logged, never leaked:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Sqlx(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
            AppError::Other(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

Rules:

- **Handlers return `Result<T, AppError>`.** Propagate `sqlx::Error` and
  `anyhow::Error` with `?`; the `#[from]` impls convert them.
- **Non-200 success goes in the `Ok` value**, e.g. `Ok((StatusCode::CREATED, Json(item)))`.
- **Map "absent" at the handler boundary.** Convert a `None` lookup with
  `.ok_or(AppError::NotFound)?`; convert a `false` mutation result (no row
  matched) to `AppError::NotFound`.
- **`BadRequest` carries a safe, human-readable message** (`"title must not be
  empty"`), shown to the caller verbatim.
- **Choose `thiserror` for the library error, `anyhow` for bootstrap.** `main`
  returns `anyhow::Result<()>`; ad-hoc internal failures flow into
  `AppError::Other`.
- **Never silently discard an error.** Propagate, map, or log it -- do not swallow
  a `Result` with `let _ =` without a written reason.

## Function Signatures

- **Parameter order:** the resource first -- `pool: &SqlitePool` for a domain
  function, `State(state): State<AppState>` for a handler -- then primary data
  (path id, payload), then options (limit, filter):

  ```rust
  pub async fn list(
      pool: &SqlitePool,
      status: Option<PostStatus>,
      limit: i64,
      offset: i64,
  ) -> Result<Vec<Post>, sqlx::Error>
  ```

- **Take what you need, no more.** `&str` for borrowed lookups (`id: &str`),
  owned `String` only when the value is stored. Return the narrowest useful type.
- **Prefix unused parameters with `_`** to document intent and silence warnings.
- **Generics stay simple.** App Starter is generics-light; when a bound is
  needed, prefer `impl Trait` in argument position with a `where` clause for
  multiple bounds. Do not introduce generics or traits where a concrete function
  is clearer -- there are no repository traits here (see Domain and Query
  Functions).

## Domain and Query Functions

Domain queries are plain free functions -- no repository struct, no trait, no
generic data-access layer. The signature shape is
`async fn name(pool: &SqlitePool, ...) -> Result<T, sqlx::Error>`.

```rust
pub async fn list(pool: &SqlitePool) -> Result<Vec<Item>, sqlx::Error> {
    sqlx::query_as::<_, Item>(
        "SELECT id, title, done, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Returns true when a row was updated, false when the id does not exist.
pub async fn toggle(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE items SET done = NOT done WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
```

- **Name for the operation, not the sqlx call:** `list`, `get`, `create`,
  `toggle`, `remove` (items); `list`, `get`, `create`, `publish`, `archive`,
  `stats` (posts).
- **Return types follow the operation:** `Vec<T>` for lists (`fetch_all`),
  `Option<T>` for single-row lookups (`fetch_optional`), `T` for creates,
  `bool` (from `result.rows_affected() > 0`) for by-id mutations so the handler
  can tell "no such row" from "done".
- **Always bind parameters** with `.bind(...)` and positional placeholders
  (`?1`, `?2`). Never interpolate user input into SQL. Where a column list is
  reused, factor it into a `const &str` and concatenate only trusted fragments:

  ```rust
  const SELECT_COLUMNS: &str =
      "SELECT id, title, body, status, created_at, published_at FROM posts";
  ```

- **Domain functions trust their inputs.** Trimming, emptiness checks, enum
  parsing, and range clamping belong in the handler, before these are called.

## HTTP Handlers

Handlers are thin: extract typed inputs, validate at the HTTP boundary, call the
domain function, wrap the result. Take `State(state): State<AppState>` for the
pool plus `Path`, `Query`, or `Json` as needed.

```rust
#[utoipa::path(
    post,
    path = "/api/v1/todo",
    tag = "items",
    request_body = CreateItem,
    responses(
        (status = 201, description = "Item created", body = Item),
        (status = 400, description = "Empty title")
    )
)]
pub async fn create_item(
    State(state): State<AppState>,
    Json(body): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), AppError> {
    let title = body.title.trim().to_owned();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".into()));
    }
    let item = items::create(&state.pool, title).await?;
    Ok((StatusCode::CREATED, Json(item)))
}
```

**Query parameters** get a `Deserialize` struct deriving `utoipa::IntoParams`,
with `#[serde(default = "fn")]` for custom defaults; clamp numeric inputs in the
handler:

```rust
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListPostsQuery {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
// ...
let posts = posts::list(&state.pool, status, query.limit.clamp(1, 100), query.offset.max(0)).await?;
```

**State transitions** fetch the entity, check the precondition (`BadRequest` on
failure), run the conditional update, and confirm it matched (`NotFound` if not).
`publish_post` is the reference:

```rust
let post = posts::get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
if post.status != PostStatus::Draft {
    return Err(AppError::BadRequest("only draft posts can be published".into()));
}
if !posts::publish(&state.pool, &id).await? {
    return Err(AppError::NotFound);
}
```

A handler that returns one of two status codes from the same body type may return
`Response` directly instead of `Result`, as `health` does.

## OpenAPI and the Typegen Loop

The OpenAPI document is the single source of truth for the frontend's types.
Every handler carries `#[utoipa::path(...)]` with the HTTP method, the **full
literal path** under the plugin's `/api/v1/<name>` prefix, a per-plugin `tag`, any
`params(...)` and `request_body`, and a `responses(...)` entry for every status
code with its `body =` type, in axum 0.8 brace syntax (`/api/v1/todo/{id}`).

Registration is **automatic**: a plugin contributes its routes AND their OpenAPI
paths/schemas from one `utoipa-axum` declaration, so they cannot drift —

```rust
fn api(&self) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_todos, create_todo)) // route + path + schemas, together
        .routes(routes!(delete_todo))
}
```

The host folds every registered plugin's `api()` into the server router and builds
the served spec from that same router (`split_for_parts()`), so there is no
separate `paths(...)`/`components(...)` list to keep in sync — the old "three
places" footgun is gone by construction. Prefix each `ToSchema` component with
`#[schema(as = <name>_Type)]` (e.g. `todo_Todo`) so plugins can't collide. After
changing handlers, params, payloads, or schemas, run `just typegen` to regenerate
`interface/src/api/schema.d.ts`; **never hand-edit that file**. CI fails on drift
via `just check-typegen`; `openapi_spec_has_no_dangling_schema_refs` catches a
dangling `$ref`, and `typegen_spec_matches_server` plus the §6 plugin guards
(`tests/plugins.rs`) catch registry and namespacing regressions.

**Versioning.** Application routes live under `/api/v1`. Within a major version,
changes are additive only -- add fields or endpoints, never remove or repurpose
them, because the generated client is pinned to the contract. A breaking change
graduates to `/api/v2` alongside `/api/v1`. Operational endpoints
(`/api/health`, `/api/openapi.json`) stay unversioned so probes and tooling keep
a stable path.

## Building AI and Agent Features

App Starter is an AI-native foundation: it is meant to grow into intelligent,
agentic ("Software 3.0") applications, and to be built by AI coding agents as much
as by people. **No LLM or agent runtime ships in the template today** -- this
section is the forward-looking design contract for when you add one, so the
capability grows the App Starter way rather than as a bolt-on.

When you add AI/agent features, keep them clean-room and native to this template:

- **Use the latest, most capable Claude models** for agent reasoning, and make
  the model id a configuration value (env / clap) rather than hardcoding one.
- **Expose agent tools through the same typed contract.** An agent "tool" is an
  endpoint: define it under `/api/v1`, annotate it with `#[utoipa::path]`, and
  type its input and output with `ToSchema`. The OpenAPI -> TypeScript typegen
  loop then covers tool surfaces exactly as it covers CRUD, so the frontend and
  any external caller stay type-checked against one source of truth.
- **Prefer MCP (Model Context Protocol)** for tool and integration boundaries;
  keep tool inputs and outputs as `ToSchema` types so they are self-describing.
- **Reuse the existing async spine** -- tokio, graceful shutdown, spawn-and-log --
  for long-running or background agent work. Do not introduce an actor framework
  or bespoke channel topology before the code demonstrably needs it (see Async
  and Graceful Shutdown).
- **Apply the existing safety layers to agent-triggered work:** redact secrets
  from logs, keep the body-size and timeout limits, and never leak internal error
  detail back to a caller or a model (see Security and Error Handling).
- **Stay clean-room and MIT-compatible.** Write agent code from documented
  behavior, free of third-party product names and license-incompatible or
  copied frameworks (see Dependencies and Licensing).

Treat this as the standard new agent capabilities must meet, and update it -- with
worked examples -- once an agent surface actually lands in the template.

## Cross-Cutting Layers

Concerns that apply to every request are tower layers on the router, not code
repeated per handler. Limits are module-level `const` values so they are central
and auditable.

```rust
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        // ... routes ...
        .fallback(crate::frontend::spa)
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, REQUEST_TIMEOUT))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}
```

- **Layer order matters.** Layers apply outermost-first in reverse of call order,
  so `SetRequestIdLayer` is added last to be outermost, ensuring the trace layer
  and handlers see the request id. Preserve this ordering.
- **Permissive CORS** exists so the desktop shell can call the local API; tighten
  it before exposing the API publicly (see Security).
- **Keep `/api/health` cheap.** It runs a trivial `SELECT 1` readiness check and
  returns 503 when the database is unreachable.

## Database and Migrations

SQLite through sqlx, single-writer; keep write transactions short. `db::init`
creates the parent directory for a file-backed database, connects, and runs
migrations:

```rust
pub async fn init(database_url: &str) -> anyhow::Result<SqlitePool> {
    // ...create parent dir for a file-backed db...
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

**Migrations are append-only.** Files are `YYYYMMDDHHMMSS_<desc>.sql`, applied in
lexicographic order. Never edit or rename a committed migration -- sqlx checksums
every applied file and the app refuses to start if one changes. To evolve the
schema, add a new file that sorts last; to fix a bad migration, add a corrective
forward one (there are no down-migrations).

Column conventions:

```sql
CREATE TABLE posts (
    id TEXT PRIMARY KEY,                       -- uuid generated app-side
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL,                   -- RFC 3339; required
    published_at TEXT                           -- nullable optional timestamp
);
CREATE INDEX idx_posts_status_created_at ON posts (status, created_at);
```

- `TEXT PRIMARY KEY` for ids, not `AUTOINCREMENT`.
- `INTEGER NOT NULL DEFAULT 0` for booleans.
- `TEXT` for timestamps; `NOT NULL` for required, nullable for optional.
- Index any column you sort or filter by; for combined filter-and-sort, a
  composite index in filter-then-sort order (`(status, created_at)`).

## Async and Graceful Shutdown

The runtime is tokio. `main` is annotated `#[tokio::main]`, returns
`anyhow::Result<()>`, builds the pool and router, binds a `TcpListener`, and
serves with graceful shutdown:

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

Shutdown awaits Ctrl-C or SIGTERM via `tokio::select!`:

```rust
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received, draining connections");
}
```

Draining matters for SQLite: a write interrupted mid-flight can leave the
database locked. If you spawn independent work with `tokio::spawn`, do not let its
errors vanish -- log failures inside the task or join the handle. Use axum 0.8
brace syntax for path params and keep the route and `#[utoipa::path]` strings in
sync.

## Pattern Matching

- **`let-else` for early returns.** Bind the happy path and bail otherwise, as
  `db::redact_url` does:

  ```rust
  let Some((scheme, rest)) = database_url.split_once("://") else {
      return database_url.to_owned();
  };
  ```

- **Match exhaustively.** List every variant so adding one is a compile error,
  not a silent fallthrough; `AppError`'s `IntoResponse` and `PostStatus::parse`
  match every case. The stats fold decodes the grouped column straight into
  `PostStatus` precisely so its match needs no `_` arm -- a new lifecycle state
  then fails to compile here instead of being silently dropped from the counts:

  ```rust
  let rows: Vec<(PostStatus, i64)> = /* SELECT status, COUNT(*) ... GROUP BY status */;
  for (status, count) in rows {
      match status {
          PostStatus::Draft => stats.draft = count,
          PostStatus::Published => stats.published = count,
          PostStatus::Archived => stats.archived = count,
      }
  }
  ```

  Reserve a `_ =>` arm for genuine indifference, not for a closed set you own.

- **Destructure in the arm** to pull out just the fields you need.

## Constants

- **Module-level `const` for limits and fixed strings**, named and centralized so
  they are auditable: `MAX_BODY_BYTES` and `REQUEST_TIMEOUT` in `src/api.rs`,
  `SELECT_COLUMNS` in `plugins/blog/src/lib.rs`.

  ```rust
  const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
  const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
  ```

- **A small `fn` for a serde default value** when an attribute needs one, as
  `default_limit` does for `ListPostsQuery`:

  ```rust
  fn default_limit() -> i64 { 50 }
  ```

- Document any non-obvious constant with a `///` comment that says why the value
  is what it is (see Comments). Reach for `LazyLock` only when a constant needs
  runtime initialization; nothing in the template needs it yet.

## Logging and Tracing

- **Use `tracing` for all logging**, initialized in `main` with an `EnvFilter`
  that defaults to `info` when `RUST_LOG` is unset.
- **Emit structured key-value fields, not interpolated strings:**

  ```rust
  tracing::info!(port = args.port, "listening");
  tracing::error!(error = %e, "database error");
  ```

- Log at the right level: `error` for broken, `warn` for degraded-but-continuing,
  `info` for lifecycle events, `debug`/`trace` for operational detail.
- **Never log secrets** -- connection strings are redacted before logging (see
  Security).

## Security

Security is enforced in shared places, not sprinkled per handler.

- **Never log credentials.** Connection strings pass through `db::redact_url`,
  which masks `user:password@` userinfo as `***` (and leaves the default SQLite
  URL untouched). Apply the same discipline to any value that could carry a
  secret before it reaches a log line:

  ```rust
  tracing::info!(database = %db::redact_url(&args.database_url), "listening");
  ```

- **Never leak internal detail to clients.** `AppError` maps `sqlx`/`anyhow`
  failures to a generic `500 {"error":"internal error"}` and logs the real cause
  server-side (see Error Handling). Do not return SQL text, file paths, or
  secrets in a response body.
- **Bound every request.** The `DefaultBodyLimit` and `TimeoutLayer` in the
  router cap memory and connection time so a single client cannot exhaust the
  server (see Cross-Cutting Layers). Raise a limit only with a written reason.
- **CORS is permissive by default for the desktop sidecar.** Tighten `CorsLayer`
  in `src/api.rs` before exposing the API publicly without the embedded UI.
- **Readiness over liveness.** `/api/health` confirms the database answers, so a
  half-broken instance is pulled from rotation rather than served traffic.
- **Parameterize all SQL** with `.bind(...)`; never interpolate user input (see
  Domain and Query Functions).

## Panics and Unsafe

- **Do not `unwrap()` or `expect()` in request-handling paths.** Propagate with
  `?` and map to `AppError`. A panic in a handler is a 500 with a noisy log and
  no useful client message.
- **`expect()` is for startup invariants only** -- where failure means the
  process cannot run, like installing the signal handlers. Give it a message that
  states the invariant (`"install Ctrl-C handler"`).
- **Do not use `unsafe`.** This codebase is entirely safe Rust; introducing
  `unsafe` requires explicit approval and a documented justification.

## Testing

Primary tests are black-box integration tests in `tests/api.rs`. They build the
real router against an in-memory database and drive it with `tower`'s `oneshot`.
The crate is imported as `app_starter`.

```rust
async fn test_app() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    sqlx::migrate!("./migrations").run(&pool).await.expect("run migrations");
    api::router(AppState { pool })
}
```

- **Cover happy path plus one 400 and one 404 per resource.** The `items`
  roundtrip exercises create, list, toggle, and delete; the `400`/`404` cases
  assert `StatusCode::BAD_REQUEST` and `StatusCode::NOT_FOUND`.
- **Keep the OpenAPI guard test** that verifies every `$ref` in the served spec
  resolves to a definition in `components.schemas`.
- **Unit-test pure logic where it lives**, in a `#[cfg(test)] mod tests` inside
  the domain module. `PostStatus::parse` and `db::redact_url` are the model:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn parse_accepts_known_statuses_and_rejects_everything_else() {
          assert_eq!(PostStatus::parse("draft"), Some(PostStatus::Draft));
          assert_eq!(PostStatus::parse("Published"), None);
          assert_eq!(PostStatus::parse(""), None);
      }
  }
  ```

- `.unwrap()` is acceptable in tests.

## Dependencies and Licensing

This is an MIT-licensed template, so dependency hygiene is a hard requirement
enforced by `cargo-deny` (`deny.toml`) in CI.

- **Licenses.** Every dependency must be MIT-compatible. The `deny.toml` allow
  list is MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-3-Clause, BSL-1.0,
  CDLA-Permissive-2.0, ISC, Unicode-3.0, Unlicense, and Zlib (confidence
  threshold 0.8). A `MIT OR Apache-2.0` crate passes if any option matches. Do
  not add a copyleft or otherwise incompatible dependency.
- **Advisories.** Yanked crates fail CI (`yanked = "deny"`); unmaintained crates
  are checked within the workspace. The `ignore` list is for tolerated advisories
  only, each with a documented rationale.
- **Sources.** Only crates.io is allowed; unknown registries and git sources are
  denied, as are wildcard version requirements.
- **Prefer the standard library and existing dependencies** before adding a new
  one. Major upgrades, an edition bump, or a toolchain/MSRV change need
  deliberate approval -- they ripple into the supply-chain gates and downstream
  consumers of the template.
- **Externally adapted code** must be license-compatible and free of third-party
  product or project names in identifiers, comments, or strings. Prefer a
  clean-room reimplementation from documented behavior over copying source.

## Comments

- **Explain why, not what.** The code says what it does; comments capture intent
  and the non-obvious reasoning (why a layer is ordered a certain way, why a body
  limit exists).
- **Doc comments (`///`, `//!`) on public items, modules, and non-obvious
  constants.** A reader should understand a public function from its doc comment
  without reading the body.
- **Write in a timeless, neutral voice.** No changelog-style comments
  ("changed X"), no decorative section-divider banners, no inline ticket or
  author references. History belongs in version control.
- **No emojis** anywhere in code, comments, or documentation.
