//! Black-box API tests: build the router against an in-memory SQLite
//! database and drive it with tower's `oneshot`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use std::collections::BTreeSet;
use tower::util::ServiceExt;

use app_starter::{AppState, api};

mod common;

/// Builds the router AND hands back the pool, so tests that need to manipulate
/// the database directly (seed many rows, or close the pool to simulate an
/// outage) can do so. `test_app` wraps this for the common router-only case.
async fn test_app_with_pool() -> (Router, SqlitePool) {
    let pool = common::memory_pool().await;
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

/// Route<->spec parity, registry-walk form (docs/plugin-framework.md §6). The
/// router and the OpenAPI spec are now built from the SAME `api_router()` and
/// separated with `split_for_parts()`, so every route and its spec path come from
/// one `#[utoipa::path]` declaration. The old source-text parser (and its
/// `.nest`/`.merge` ban) is gone: `.merge` is now the registration mechanism, and
/// the "served but undocumented" and "deleted route shadowed by a parameterized
/// sibling" classes are impossible by construction -- a route cannot be registered
/// without contributing its spec path -- so [review M9] those source-level checks
/// are dropped, not hidden. What remains is the runtime property: every operation
/// the served spec declares is actually reachable on the router.
///
/// "Not served" is detected via the SPA fallback rather than status alone (which
/// avoids 404-vs-405 flakiness): an unmatched path falls through to
/// `crate::frontend::spa`, which serves index.html (HTML content-type) or, when
/// `interface/dist` is empty -- the state under `just test`, which sets
/// SKIP_FRONTEND_BUILD=1 -- a 404 with a "frontend not built" marker. A real
/// handler answering a probe with 404/400/415 is therefore still "served".
#[tokio::test]
async fn routes_and_openapi_spec_are_in_parity() {
    // The REST verbs the resource recipe uses. Spec operations are filtered to
    // these so a non-method path-item key (or an exotic verb) cannot desync the
    // two sides. A future custom `.head`/`.options`/`.trace` handler would fall
    // outside this set and is a known blind spot.
    const METHODS: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

    let app = test_app().await;

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

    // Every operation the spec declares is actually served by the router.
    let mut unserved: Vec<(String, String)> = Vec::new();
    for (method, path) in &spec_ops {
        let concrete = concrete_path(path);
        let (status, is_spa_fallback) = probe(&app, method, &concrete).await;
        if is_spa_fallback || status == StatusCode::METHOD_NOT_ALLOWED {
            unserved.push((method.clone(), path.clone()));
        }
    }
    assert!(
        unserved.is_empty(),
        "the OpenAPI spec declares operations the router does not serve: {unserved:?}"
    );
}

/// [review M2] The spec `just typegen` consumes (printed by the `openapi_spec`
/// bin via `api::api_spec()`) must equal the spec the server serves at
/// `/api/openapi.json` (served by `openapi_json`). Both derive from the same
/// `api_router()`, so this holds by construction; the guard fails loudly if a
/// future change makes the bin and the handler build their specs differently and
/// silently desyncs the generated `schema.d.ts` from what is served.
#[tokio::test]
async fn typegen_spec_matches_server() {
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
    let served = body_json(response).await;

    let typegen: serde_json::Value =
        serde_json::to_value(api::api_spec()).expect("serialize api_spec");

    assert_eq!(
        served, typegen,
        "served /api/openapi.json differs from the typegen spec (api::api_spec())"
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
/// route can be probed (`/api/v1/todo/{id}` -> `/api/v1/todo/__parity_probe__`).
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
async fn todo_crud_roundtrip() {
    let app = test_app().await;

    // Create
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/todo")
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
        .oneshot(Request::get("/api/v1/todo").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Toggle
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/todo/{id}/toggle"))
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
            Request::delete(format!("/api/v1/todo/{id}"))
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
            Request::post("/api/v1/todo")
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
            Request::delete("/api/v1/todo/does-not-exist")
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
            Request::post("/api/v1/todo")
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

/// Write-side guard: the posts table carries
/// `CHECK (status IN ('draft','published','archived'))` (migration
/// 20260621000001), so the database itself rejects an out-of-vocabulary status.
/// No path -- API, seed, or raw SQL -- can persist a status the `PostStatus`
/// enum cannot represent. A valid status through the same statement still
/// succeeds, proving the rejection is the CHECK rejecting the vocabulary rather
/// than a malformed INSERT.
#[tokio::test]
async fn out_of_vocabulary_status_is_rejected_by_the_check_constraint() {
    let (_app, pool) = test_app_with_pool().await;

    let rejected =
        sqlx::query("INSERT INTO posts (id, title, status, created_at) VALUES (?1, ?2, ?3, ?4)")
            .bind("bad-row")
            .bind("nope")
            .bind("scheduled")
            .bind("2026-06-21T00:00:00Z")
            .execute(&pool)
            .await;
    assert!(
        rejected.is_err(),
        "CHECK (status IN (...)) must reject an out-of-vocabulary status, got {rejected:?}"
    );

    sqlx::query("INSERT INTO posts (id, title, status, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind("good-row")
        .bind("ok")
        .bind("draft")
        .bind("2026-06-21T00:00:00Z")
        .execute(&pool)
        .await
        .expect("a valid status must insert cleanly through the same statement");
}

/// Read-side guard, defense in depth behind the CHECK constraint: if a row
/// outside the lifecycle vocabulary ever exists anyway -- legacy data written
/// before the constraint, a manual edit, or a path that bypasses it -- every
/// read decodes `status` into `PostStatus`, so it surfaces as a loud 500 rather
/// than being silently dropped (issue #47 removed the old `_ => {}` arm). The
/// blast radius is the whole read surface, not just stats: the unfiltered list
/// 500s too. We reproduce such a row by disabling CHECK enforcement on a single
/// connection (`PRAGMA ignore_check_constraints`) -- the only way to seed a row
/// the constraint would otherwise forbid.
#[tokio::test]
async fn out_of_vocabulary_legacy_row_fails_loudly_on_every_read_path() {
    let (app, pool) = test_app_with_pool().await;

    // Seed the forbidden row on one connection with CHECK enforcement off, then
    // release it so the requests below can run. test_app_with_pool builds a
    // `max_connections(1)` pool, so every acquire reuses the same physical
    // connection and its `sqlite::memory:` database -- that single reused
    // connection (a bare in-memory DB is otherwise per-connection) is why the
    // seeded row is visible to these reads, as in the other seed-via-pool tests.
    {
        let mut conn = pool.acquire().await.expect("acquire connection");
        sqlx::query("PRAGMA ignore_check_constraints = ON")
            .execute(&mut *conn)
            .await
            .expect("disable check enforcement");
        sqlx::query("INSERT INTO posts (id, title, status, created_at) VALUES (?1, ?2, ?3, ?4)")
            .bind("legacy-row")
            .bind("legacy")
            .bind("scheduled")
            .bind("2026-06-21T00:00:00Z")
            .execute(&mut *conn)
            .await
            .expect("seed an out-of-vocabulary status row");
        sqlx::query("PRAGMA ignore_check_constraints = OFF")
            .execute(&mut *conn)
            .await
            .expect("restore check enforcement");
    }

    // Both the aggregate and the unfiltered list decode the poisoned row, so each
    // fails loud (AppError::Sqlx -> 500) instead of returning 200 with it dropped.
    for path in ["/api/v1/posts/stats", "/api/v1/posts"] {
        let response = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{path} must fail loud on an out-of-vocabulary row",
        );
    }
}
