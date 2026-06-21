//! Black-box API tests: build the router against an in-memory SQLite
//! database and drive it with tower's `oneshot`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
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
