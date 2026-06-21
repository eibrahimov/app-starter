//! Black-box API tests: build the router against an in-memory SQLite
//! database and drive it with tower's `oneshot`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::BTreeSet;
use tower::util::ServiceExt;

use app_starter::{AppState, api};

/// Builds the router AND hands back the pool, so tests that need to manipulate
/// the database directly (seed many rows, or close the pool to simulate an
/// outage) can do so. `test_app` wraps this for the common router-only case.
async fn test_app_with_pool() -> (Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    let router = api::router(AppState { pool: pool.clone() });
    (router, pool)
}

async fn test_app() -> Router {
    test_app_with_pool().await.0
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["status"], "ok");
    // Readiness probe also confirms the database is reachable.
    assert_eq!(json["database"], "ok");
}

/// Guards the typegen loop's biggest footgun: a handler is registered in
/// `paths(...)` but its request/response type is missing from
/// `components(schemas(...))` in src/api.rs. That produces a dangling
/// `$ref` in the served spec, which silently breaks the generated TypeScript.
/// This fails CI before the broken types reach the frontend.
#[tokio::test]
async fn openapi_spec_has_no_dangling_schema_refs() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let spec = body_json(response).await;

    let defined: std::collections::HashSet<String> = spec
        .pointer("/components/schemas")
        .and_then(|s| s.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();

    let mut referenced = Vec::new();
    collect_schema_refs(&spec, &mut referenced);

    let dangling: Vec<&String> = referenced
        .iter()
        .filter(|r| !defined.contains(*r))
        .collect();
    assert!(
        dangling.is_empty(),
        "OpenAPI references schemas missing from components(schemas(...)) in \
         src/api.rs: {dangling:?}"
    );
}

/// Route<->spec parity guard (issue #32). The OpenAPI spec is generated from the
/// `#[openapi(paths(...))]` list in src/api.rs, while the router is built from a
/// separate set of `.route(...)` calls. A handler can be wired with `.route(...)`
/// but omitted from `paths(...)` -- it compiles, then silently vanishes from the
/// spec and the generated TypeScript client -- or listed in the spec with no
/// matching route. Neither is a compile error, and `openapi_spec_has_no_dangling_\
/// schema_refs` only catches a missing *schema*, not a missing *path*. This test
/// asserts the route surface three ways:
///   1. every operation the spec declares is actually served by the router,
///   2. every `/api/v1` operation the router serves is declared in the spec, and
///   3. the set of `/api/v1` `.route(...)` path literals in src/api.rs equals the
///      set of `/api/v1` spec paths -- a source-level check that still catches a
///      deleted or renamed route when a parameterized sibling (e.g.
///      `/api/v1/posts/{id}`) would shadow the missing path at runtime and hide it
///      from check 1.
///
/// "Not served" is detected via the SPA fallback rather than status alone (which
/// avoids 404-vs-405 flakiness): an unmatched path falls through to
/// `crate::frontend::spa`, which serves index.html (HTML content-type) or, when
/// `interface/dist` is empty -- the state under `just test`, which sets
/// SKIP_FRONTEND_BUILD=1 -- a 404 with a "frontend not built" marker. A real
/// handler answering a probe with 404/400/415 is therefore still "served".
#[tokio::test]
async fn routes_and_openapi_spec_are_in_parity() {
    // The REST verbs the resource recipe uses. Probing is restricted to these;
    // spec operations are filtered to them so a non-method path-item key (or an
    // exotic verb) cannot desync the two sides. A future custom `.head`/`.options`/
    // `.trace` handler would fall outside this set and is a known blind spot.
    const METHODS: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

    let app = test_app().await;

    // --- the (method, path) operations the spec declares ---
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let spec = body_json(response).await;
    let paths = spec
        .pointer("/paths")
        .and_then(|paths| paths.as_object())
        .expect("served spec has a /paths object");

    let mut spec_ops: BTreeSet<(String, String)> = BTreeSet::new();
    for (path, item) in paths {
        let Some(operations) = item.as_object() else {
            continue;
        };
        for method in operations.keys() {
            let method = method.to_uppercase();
            if METHODS.contains(&method.as_str()) {
                spec_ops.insert((method, path.clone()));
            }
        }
    }
    assert!(
        !spec_ops.is_empty(),
        "the served spec declared no operations -- the test harness is broken"
    );

    // --- the `/api/v1` paths the source actually wires with `.route(...)` ---
    // axum exposes no API to enumerate a built router's routes, and refactoring
    // the router into a data table is out of scope (this is a test), so the
    // source is read as the source of truth for "what the router serves". These
    // are the candidate paths for finding a served-but-undocumented route; the
    // spec paths cover the opposite direction.
    let api_source = include_str!("../src/api.rs");
    // The parser below treats `.route("literal", ...)` calls as the source of
    // truth for what the router serves. Routes added via `.nest(`/`.merge(` are
    // invisible to it, which would silently shrink checks 2 and 3 -- so fail
    // loudly if the router ever moves off the flat style this parser assumes.
    assert!(
        !api_source.contains(".nest(") && !api_source.contains(".merge("),
        "src/api.rs uses `.nest(`/`.merge(`, which `declared_v1_route_paths` \
         cannot see -- those routes vanish from checks 2 and 3. Flatten the \
         router into `.route(...)` calls or teach the parser the new shape."
    );
    let source_v1_routes = declared_v1_route_paths(api_source);
    assert!(
        !source_v1_routes.is_empty(),
        "parsed no /api/v1 `.route(...)` literals from src/api.rs -- the parser \
         or the source layout changed"
    );

    // --- the (method, path) operations the router actually serves ---
    let mut candidate_paths: BTreeSet<String> =
        spec_ops.iter().map(|(_, path)| path.clone()).collect();
    candidate_paths.extend(source_v1_routes.iter().cloned());

    let mut served_ops: BTreeSet<(String, String)> = BTreeSet::new();
    for path in &candidate_paths {
        let concrete = concrete_path(path);
        for method in METHODS {
            let (status, is_spa_fallback) = probe(&app, method, &concrete).await;
            if is_spa_fallback || status == StatusCode::METHOD_NOT_ALLOWED {
                continue;
            }
            served_ops.insert((method.to_owned(), path.clone()));
        }
    }

    // Direction 1: every operation the spec declares is served. A spec entry with
    // no matching route means a `#[utoipa::path]` handler was added to
    // `#[openapi(paths(...))]` without a matching `.route(...)` in src/api.rs.
    let declared_but_unserved: Vec<&(String, String)> = spec_ops.difference(&served_ops).collect();
    assert!(
        declared_but_unserved.is_empty(),
        "OpenAPI declares operations the router does not serve -- add the missing \
         `.route(...)` in src/api.rs: {declared_but_unserved:?}"
    );

    // Direction 2: every `/api/v1` operation the router serves is in the spec.
    // This is the silent footgun: a handler wired with `.route(...)` but omitted
    // from `#[openapi(paths(...))]` vanishes from the spec and the generated
    // TypeScript client. Scoped to `/api/v1` so the intentionally-undocumented
    // operational routes (`/api/openapi.json`) are not flagged.
    let served_but_undocumented: Vec<&(String, String)> = served_ops
        .iter()
        .filter(|(_, path)| path.starts_with("/api/v1"))
        .filter(|operation| !spec_ops.contains(*operation))
        .collect();
    assert!(
        served_but_undocumented.is_empty(),
        "the router serves /api/v1 operations absent from the OpenAPI spec -- add \
         the handler to `#[openapi(paths(...))]` in src/api.rs or it vanishes from \
         the generated TS client: {served_but_undocumented:?}"
    );

    // Direction 3 (source vs spec, path level): the `/api/v1` paths wired with
    // `.route(...)` in src/api.rs must match the `/api/v1` paths in the served
    // spec. Unlike check 1 this is a source comparison, not a runtime probe, so it
    // catches a deleted or renamed route even when a parameterized sibling shadows
    // the missing path -- e.g. dropping `.route("/api/v1/posts/stats", ...)` while
    // `posts::post_stats` stays in `paths(...)`: a probe to `/api/v1/posts/stats`
    // would match `/api/v1/posts/{id}` and look "served", but the literal is gone.
    let spec_v1_paths: BTreeSet<String> = spec_ops
        .iter()
        .map(|(_, path)| path.clone())
        .filter(|path| path.starts_with("/api/v1"))
        .collect();
    let routed_but_unspecified: Vec<&String> =
        source_v1_routes.difference(&spec_v1_paths).collect();
    let specified_but_unrouted: Vec<&String> =
        spec_v1_paths.difference(&source_v1_routes).collect();
    assert!(
        routed_but_unspecified.is_empty() && specified_but_unrouted.is_empty(),
        "route<->spec /api/v1 path drift in src/api.rs -- the `.route(...)` literals \
         and the OpenAPI `/api/v1` paths must be identical: routed but absent from \
         spec = {routed_but_unspecified:?}; in spec but not routed = \
         {specified_but_unrouted:?}"
    );
}

/// Sends `method path` through the router and reports `(status, is_spa_fallback)`.
/// `is_spa_fallback` is true when no API route matched and the request fell
/// through to `crate::frontend::spa`: a served index.html (HTML content-type) or,
/// when `interface/dist` is empty, a 404 whose body is the "frontend not built"
/// marker. A real handler -- even one returning 404/400/415 for a probe with a
/// dummy id and empty body -- is not the fallback.
async fn probe(app: &Router, method: &str, path: &str) -> (StatusCode, bool) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let is_spa_fallback = content_type.contains("text/html")
        || String::from_utf8_lossy(&bytes).starts_with("frontend not built");
    (status, is_spa_fallback)
}

/// Replaces every `{param}` path segment with a dummy value so a parameterized
/// route can be probed (`/api/v1/items/{id}` -> `/api/v1/items/__parity_probe__`).
/// The sentinel is deliberately improbable so it cannot collide with a real static
/// route (e.g. `/api/v1/posts/stats`) and mask a missing parameterized route.
fn concrete_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                "__parity_probe__"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Extracts the `/api/v1/...` path literals from the `.route(...)` calls in
/// src/api.rs source text. Only the string literal that is the first argument of
/// `.route(` is taken, so prose mentioning `/api/v1` is ignored; a `.route(` that
/// sits behind a `//` line comment (commented-out or example code) is skipped, so
/// it cannot inject a phantom route. A `.route(` with no string literal before the
/// next `.route(` (e.g. a `const` path) is passed over rather than mis-paired with
/// a later route's quote, and one malformed call never aborts the rest of the
/// scan. Assumes the flat-router style this template uses: one
/// `.route("literal", ...)` per statement and no block comment wrapping a route.
/// The caller separately asserts src/api.rs has no `.nest(`/`.merge(`, which this
/// parser cannot see.
fn declared_v1_route_paths(source: &str) -> BTreeSet<String> {
    let mut routes = BTreeSet::new();
    let mut search_from = 0usize;
    while let Some(relative) = source[search_from..].find(".route(") {
        let route_at = search_from + relative;
        let after_token = route_at + ".route(".len();
        // Advance unconditionally, so a malformed or non-literal `.route(` is
        // skipped rather than aborting the scan of everything after it.
        search_from = after_token;
        // Skip a `.route(` that sits behind a `//` on its own line.
        let line_start = source[..route_at].rfind('\n').map_or(0, |nl| nl + 1);
        if source[line_start..route_at].contains("//") {
            continue;
        }
        // The path literal is the first argument and must appear before the next
        // `.route(`; otherwise this call has no string-literal first arg, so skip
        // it instead of stealing a later route's quote.
        let next_route = source[after_token..]
            .find(".route(")
            .map_or(source.len(), |rel| after_token + rel);
        let segment = &source[after_token..next_route];
        let Some(open_rel) = segment.find('"') else {
            continue;
        };
        let open = after_token + open_rel + 1;
        let Some(close_rel) = source[open..next_route].find('"') else {
            continue;
        };
        let path = &source[open..open + close_rel];
        if path.starts_with("/api/v1") {
            routes.insert(path.to_owned());
        }
    }
    routes
}

/// Collects every `#/components/schemas/<Name>` referenced anywhere in the spec.
fn collect_schema_refs(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "$ref"
                    && let Some(name) = val
                        .as_str()
                        .and_then(|s| s.strip_prefix("#/components/schemas/"))
                {
                    out.push(name.to_owned());
                } else {
                    collect_schema_refs(val, out);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_schema_refs(item, out);
            }
        }
        _ => {}
    }
}

#[tokio::test]
async fn items_crud_roundtrip() {
    let app = test_app().await;

    // Create
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"first item"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["done"], false);

    // List
    let response = app
        .clone()
        .oneshot(Request::get("/api/v1/items").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Toggle
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/items/{id}/toggle"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let toggled = body_json(response).await;
    assert_eq!(toggled["done"], true);

    // Delete
    let response = app
        .clone()
        .oneshot(
            Request::delete(format!("/api/v1/items/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Empty title rejected
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Unknown id is a 404
    let response = app
        .oneshot(
            Request::delete("/api/v1/items/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn posts_lifecycle_roundtrip() {
    let app = test_app().await;

    // Create lands as a draft
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/posts")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"hello world","body":"first post"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["status"], "draft");
    assert!(created["published_at"].is_null());

    // Stats count the draft
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/posts/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let stats = body_json(response).await;
    assert_eq!(stats["draft"], 1);
    assert_eq!(stats["published"], 0);

    // Get by id
    let response = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/posts/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Publish sets status and timestamp
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/posts/{id}/publish"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let published = body_json(response).await;
    assert_eq!(published["status"], "published");
    assert!(published["published_at"].is_string());

    // Publishing twice is an invalid transition
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/posts/{id}/publish"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Status filter narrows the list
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/posts?status=published")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/posts?status=draft")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 0);

    // Archive completes the lifecycle
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/posts/{id}/archive"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let archived = body_json(response).await;
    assert_eq!(archived["status"], "archived");
}

#[tokio::test]
async fn post_invalid_input_rejected() {
    let app = test_app().await;

    // Empty title is a 400
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/posts")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Unknown status filter is a 400
    let response = app
        .oneshot(
            Request::get("/api/v1/posts?status=bogus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_unknown_id_is_404() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::post("/api/v1/posts/does-not-exist/publish")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// --- Cross-cutting layer + error-path coverage -----------------------------
// These lock down the production-minded behaviors README/docs advertise but
// that the happy-path tests above never exercise, so a refactor can't silently
// break them. The 408 timeout path (TimeoutLayer in src/api.rs) is intentionally
// not unit-tested: triggering it would require a deliberately slow handler that
// does not belong in the template; its status mapping is asserted by config.

/// The router tags every response with an `x-request-id` so a client failure
/// can be correlated with the server log. An incoming id is preserved;
/// otherwise the server generates one. (Set/Propagate layers in src/api.rs.)
#[tokio::test]
async fn request_id_is_present_and_echoed() {
    let app = test_app().await;

    // No incoming id: the server generates and returns one.
    let response = app
        .clone()
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let generated = response
        .headers()
        .get("x-request-id")
        .expect("response carries an x-request-id");
    assert!(!generated.to_str().unwrap().is_empty());

    // Incoming id: the server echoes it back unchanged.
    let response = app
        .oneshot(
            Request::get("/api/health")
                .header("x-request-id", "test-correlation-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("x-request-id").unwrap(),
        "test-correlation-123"
    );
}

/// Bodies larger than `MAX_BODY_BYTES` (10 MiB) are rejected by DefaultBodyLimit
/// with 413 before any handler runs. 413 bypasses AppError, so the body is
/// axum's default rather than the `{"error":...}` envelope -- assert status only.
#[tokio::test]
async fn oversized_body_is_rejected_with_413() {
    let app = test_app().await;
    // A body over the 10 MiB cap is rejected before any handler parses it, so it
    // need not be valid JSON -- a single 11 MiB allocation is enough.
    let payload = "a".repeat(11 * 1024 * 1024);
    let response = app
        .oneshot(
            Request::post("/api/v1/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

/// The readiness probe returns 503 when the database is unreachable, so an
/// orchestrator stops routing traffic to an instance with a dead database.
#[tokio::test]
async fn health_returns_503_when_database_is_down() {
    let (app, pool) = test_app_with_pool().await;
    pool.close().await;
    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = body_json(response).await;
    assert_eq!(json["status"], "degraded");
    assert_eq!(json["database"], "unreachable");
}

/// `limit` is clamped to 1..=100 and `offset` to >= 0, so a client cannot
/// request an unbounded page or a negative offset (src/api/posts.rs).
#[tokio::test]
async fn list_posts_clamps_pagination() {
    let (app, pool) = test_app_with_pool().await;

    // Seed 101 posts directly through the pool (faster than 101 HTTP calls).
    for i in 0..101 {
        app_starter::posts::create(&pool, format!("post {i}"), String::new())
            .await
            .expect("seed post");
    }

    // A limit far above the cap returns at most 100.
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/posts?limit=100000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 100);

    // A negative offset is clamped to 0 rather than erroring.
    let response = app
        .oneshot(
            Request::get("/api/v1/posts?offset=-5&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 10);
}

/// A draft cannot be archived directly (draft -> published -> archived). The
/// invalid transition is a 400, distinct from a 404 for an unknown id.
#[tokio::test]
async fn archiving_a_draft_is_rejected() {
    let app = test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/posts")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"draft post"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let id = body_json(response).await["id"].as_str().unwrap().to_owned();

    let response = app
        .oneshot(
            Request::post(format!("/api/v1/posts/{id}/archive"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Fetching an unknown post id is a 404 on the read path (the existing
/// `post_unknown_id_is_404` covers the publish path).
#[tokio::test]
async fn get_unknown_post_is_404() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::get("/api/v1/posts/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
