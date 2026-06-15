pub mod health;
pub mod items;
pub mod posts;

use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

/// Maximum accepted request body in bytes. Generous for JSON payloads; raise it
/// (and say why) before accepting large uploads. Guards against trivial OOM/DoS.
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Per-request timeout. A request still running after this is aborted (408).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// OpenAPI document. Every handler annotated with `#[utoipa::path]` is
/// registered here; `just typegen` turns this into TypeScript types for
/// the frontend client.
#[derive(OpenApi)]
#[openapi(
    info(title = "App Starter API", description = "API for App Starter"),
    paths(
        health::health,
        items::list_items,
        items::create_item,
        items::toggle_item,
        items::delete_item,
        posts::list_posts,
        posts::create_post,
        posts::post_stats,
        posts::get_post,
        posts::publish_post,
        posts::archive_post,
    ),
    components(schemas(
        health::Health,
        crate::items::Item,
        crate::items::CreateItem,
        crate::posts::Post,
        crate::posts::CreatePost,
        crate::posts::PostStats,
    ))
)]
pub struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        // Operational endpoints stay unversioned so health probes and tooling
        // keep a stable path across API versions.
        .route("/api/health", get(health::health))
        .route("/api/openapi.json", get(openapi_json))
        // Versioned application API. Within a major version, only ADD fields or
        // endpoints -- never remove or repurpose them, since the generated
        // TypeScript client and any downstream consumers are pinned to it.
        // A breaking change graduates to /api/v2 alongside /api/v1.
        .route(
            "/api/v1/items",
            get(items::list_items).post(items::create_item),
        )
        .route("/api/v1/items/{id}", delete(items::delete_item))
        .route("/api/v1/items/{id}/toggle", post(items::toggle_item))
        .route(
            "/api/v1/posts",
            get(posts::list_posts).post(posts::create_post),
        )
        .route("/api/v1/posts/stats", get(posts::post_stats))
        .route("/api/v1/posts/{id}", get(posts::get_post))
        .route("/api/v1/posts/{id}/publish", post(posts::publish_post))
        .route("/api/v1/posts/{id}/archive", post(posts::archive_post))
        .fallback(crate::frontend::spa)
        // Permissive CORS keeps the Tauri desktop shell (tauri://localhost)
        // able to call the sidecar API. Tighten before exposing publicly.
        .layer(CorsLayer::permissive())
        // Production-minded defaults: cap body size and time out slow requests
        // so a single client cannot exhaust memory or hold a connection open.
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        // Tag every request with an x-request-id and echo it on the response,
        // so a client 500 can be correlated with the server-side error log.
        // SetRequestId is outermost (added last) so Trace and handlers see it.
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}
