pub mod health;

use crate::AppState;
use axum::Json;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::get;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// Maximum accepted request body in bytes. Generous for JSON payloads; raise it
/// (and say why) before accepting large uploads. Guards against trivial OOM/DoS.
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Per-request timeout. A request still running after this is aborted (408).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// OpenAPI document metadata only. Paths and component schemas are no longer
/// hand-listed here: each handler's `#[utoipa::path]` contributes its path AND
/// its schemas through `OpenApiRouter` (one declaration, so the router and the
/// spec cannot drift). This struct now carries just the top-level `info`.
#[derive(OpenApi)]
#[openapi(info(title = "App Starter API", description = "API for App Starter"))]
pub struct ApiDoc;

/// The application API as an `OpenApiRouter`: every route registered here also
/// contributes its OpenAPI path + schemas. Core resources (items, posts, the
/// health probe) are registered directly; every registered plugin folds in via
/// `plugins::all()` under its own `/api/v1/<name>` prefix (empty until Phase 2).
///
/// Shared HTTP layers and the SPA fallback are deliberately NOT added here -- they
/// are attached once in [`router`] after `split_for_parts()`, so plugins
/// contribute routes, never layers (docs/plugin-framework.md §4).
fn api_router() -> OpenApiRouter<AppState> {
    let mut router = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // Operational health probe stays unversioned (stable across API versions).
        // Core registers only the operational health probe; every versioned
        // application resource (/api/v1/*) is contributed by a registered plugin.
        .routes(routes!(health::health));

    for plugin in crate::plugins::all() {
        router = router.merge(plugin.api());
    }
    router
}

/// The OpenAPI spec the server serves and that `just typegen` consumes (via the
/// `openapi_spec` bin). Built from the same [`api_router`] as the live routes
/// [review M2], so the generated `schema.d.ts` cannot diverge from what is served.
pub fn api_spec() -> utoipa::openapi::OpenApi {
    api_router().split_for_parts().1
}

/// Serves the generated spec. Built fresh from [`api_spec`] so it is always the
/// exact document the router serves -- `/api/openapi.json` is tooling-only and
/// rarely hit, so rebuilding per request is fine.
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(api_spec())
}

pub fn router(state: AppState) -> Router {
    // Take the axum routes from the registry-built router; the OpenAPI spec side
    // is served separately via `openapi_json` (which rebuilds it identically).
    let router = api_router().split_for_parts().0;
    router
        // Operational endpoint stays unversioned and outside the OpenApiRouter:
        // it serves tooling, not the typed API surface.
        .route("/api/openapi.json", get(openapi_json))
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
