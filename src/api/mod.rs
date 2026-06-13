pub mod health;
pub mod items;
pub mod posts;

use crate::AppState;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

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
        .route("/api/health", get(health::health))
        .route("/api/openapi.json", get(openapi_json))
        .route(
            "/api/items",
            get(items::list_items).post(items::create_item),
        )
        .route("/api/items/{id}", delete(items::delete_item))
        .route("/api/items/{id}/toggle", post(items::toggle_item))
        .route(
            "/api/posts",
            get(posts::list_posts).post(posts::create_post),
        )
        .route("/api/posts/stats", get(posts::post_stats))
        .route("/api/posts/{id}", get(posts::get_post))
        .route("/api/posts/{id}/publish", post(posts::publish_post))
        .route("/api/posts/{id}/archive", post(posts::archive_post))
        .fallback(crate::frontend::spa)
        // Permissive CORS keeps the Tauri desktop shell (tauri://localhost)
        // able to call the sidecar API. Tighten before exposing publicly.
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
