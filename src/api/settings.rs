//! HTTP handlers for application settings: read and update the base currency.

use crate::AppState;
use crate::error::AppError;
use crate::settings::{self, Settings, UpdateSettings, valid_currency};
use axum::Json;
use axum::extract::State;

#[utoipa::path(
    get,
    path = "/api/v1/settings",
    tag = "settings",
    responses((status = 200, description = "Current settings", body = Settings))
)]
pub async fn get_settings(State(state): State<AppState>) -> Result<Json<Settings>, AppError> {
    Ok(Json(settings::get(&state.pool).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings",
    tag = "settings",
    request_body = UpdateSettings,
    responses(
        (status = 200, description = "Updated settings", body = Settings),
        (status = 400, description = "Invalid currency code")
    )
)]
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettings>,
) -> Result<Json<Settings>, AppError> {
    let code = body.base_currency.trim().to_uppercase();
    if !valid_currency(&code) {
        return Err(AppError::BadRequest(
            "base_currency must be a 3-letter code".into(),
        ));
    }
    Ok(Json(settings::update(&state.pool, &code).await?))
}
