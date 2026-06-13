use crate::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Health {
    /// "ok" when the service and its database are reachable, else "degraded".
    pub status: String,
    pub version: String,
    /// "ok" when a trivial query against the database succeeds, else "unreachable".
    pub database: String,
}

/// Readiness probe: confirms the process is up AND the database answers a
/// trivial query, so orchestrators stop routing traffic to an instance whose
/// database is gone (returns 503). A liveness-only check would report healthy
/// with a dead database and keep receiving requests.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Service and database are healthy", body = Health),
        (status = 503, description = "Database is unreachable", body = Health)
    )
)]
pub async fn health(State(state): State<AppState>) -> Response {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let (code, status, database) = if db_ok {
        (StatusCode::OK, "ok", "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded", "unreachable")
    };
    (
        code,
        Json(Health {
            status: status.into(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            database: database.into(),
        }),
    )
        .into_response()
}
