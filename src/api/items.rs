use crate::AppState;
use crate::error::AppError;
use crate::items::{self, CreateItem, Item};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

#[utoipa::path(
    get,
    path = "/api/items",
    tag = "items",
    responses((status = 200, description = "All items, newest first", body = [Item]))
)]
pub async fn list_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, AppError> {
    Ok(Json(items::list(&state.pool).await?))
}

#[utoipa::path(
    post,
    path = "/api/items",
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

#[utoipa::path(
    post,
    path = "/api/items/{id}/toggle",
    tag = "items",
    params(("id" = String, Path, description = "Item id")),
    responses(
        (status = 200, description = "Toggled item", body = Item),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn toggle_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Item>, AppError> {
    if !items::toggle(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let item = items::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(item))
}

#[utoipa::path(
    delete,
    path = "/api/items/{id}",
    tag = "items",
    params(("id" = String, Path, description = "Item id")),
    responses(
        (status = 204, description = "Item deleted"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !items::remove(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
