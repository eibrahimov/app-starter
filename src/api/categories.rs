//! HTTP handlers for categories: CRUD with name/budget validation. Deleting a
//! category uncategorizes its expenses (see `categories::remove`).

use crate::AppState;
use crate::categories::{self, Category, CreateCategory, UpdateCategory};
use crate::error::AppError;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

fn validate(name: &str, budget: Option<i64>) -> Result<String, AppError> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(AppError::BadRequest("name must not be empty".into()));
    }
    if budget.is_some_and(|b| b < 0) {
        return Err(AppError::BadRequest(
            "monthly_budget_cents must not be negative".into(),
        ));
    }
    Ok(name)
}

/// Maps the UNIQUE(name) constraint violation to a friendly 400 rather than a
/// generic 500, since a duplicate name is user input, not a server fault.
fn map_save_error(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db) = &e
        && db.is_unique_violation()
    {
        return AppError::BadRequest("a category with that name already exists".into());
    }
    AppError::Sqlx(e)
}

#[utoipa::path(
    get,
    path = "/api/v1/categories",
    tag = "categories",
    responses((status = 200, description = "All categories, A–Z", body = [Category]))
)]
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, AppError> {
    Ok(Json(categories::list(&state.pool).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/categories",
    tag = "categories",
    request_body = CreateCategory,
    responses(
        (status = 201, description = "Category created", body = Category),
        (status = 400, description = "Invalid name or budget")
    )
)]
pub async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategory>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    let name = validate(&body.name, body.monthly_budget_cents)?;
    let category = categories::create(&state.pool, name, body.color, body.monthly_budget_cents)
        .await
        .map_err(map_save_error)?;
    Ok((StatusCode::CREATED, Json(category)))
}

#[utoipa::path(
    get,
    path = "/api/v1/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    responses(
        (status = 200, description = "The category", body = Category),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Category>, AppError> {
    let category = categories::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(category))
}

#[utoipa::path(
    put,
    path = "/api/v1/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    request_body = UpdateCategory,
    responses(
        (status = 200, description = "Updated category", body = Category),
        (status = 400, description = "Invalid name or budget"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateCategory>,
) -> Result<Json<Category>, AppError> {
    let name = validate(&body.name, body.monthly_budget_cents)?;
    if !categories::update(
        &state.pool,
        &id,
        name,
        body.color,
        body.monthly_budget_cents,
    )
    .await
    .map_err(map_save_error)?
    {
        return Err(AppError::NotFound);
    }
    let category = categories::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(category))
}

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    responses(
        (status = 204, description = "Category deleted; its expenses are uncategorized"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !categories::remove(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
