//! HTTP handlers for expenses: filtered list, CRUD with amount/date validation.
//! Amounts are integer minor units (cents); a non-positive amount, a malformed
//! date, or an unknown category id are all 400s.

use crate::AppState;
use crate::error::AppError;
use crate::expenses::{self, CreateExpense, Expense, UpdateExpense};
use crate::{categories, expenses::valid_date, expenses::valid_month};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListExpensesQuery {
    /// Filter to a single month, `YYYY-MM`.
    pub month: Option<String>,
    /// Filter to a single category id.
    pub category_id: Option<String>,
}

/// Shared validation for create/update. Returns the cleaned description.
async fn validate(
    state: &AppState,
    amount_cents: i64,
    spent_on: &str,
    category_id: &Option<String>,
) -> Result<(), AppError> {
    if amount_cents <= 0 {
        return Err(AppError::BadRequest(
            "amount_cents must be a positive integer".into(),
        ));
    }
    if !valid_date(spent_on) {
        return Err(AppError::BadRequest("spent_on must be YYYY-MM-DD".into()));
    }
    if let Some(id) = category_id
        && categories::get(&state.pool, id).await?.is_none()
    {
        return Err(AppError::BadRequest("unknown category_id".into()));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses",
    tag = "expenses",
    params(ListExpensesQuery),
    responses(
        (status = 200, description = "Expenses, newest first", body = [Expense]),
        (status = 400, description = "Malformed month filter")
    )
)]
pub async fn list_expenses(
    State(state): State<AppState>,
    Query(query): Query<ListExpensesQuery>,
) -> Result<Json<Vec<Expense>>, AppError> {
    if let Some(month) = &query.month
        && !valid_month(month)
    {
        return Err(AppError::BadRequest("month must be YYYY-MM".into()));
    }
    let expenses = expenses::list(
        &state.pool,
        query.month.as_deref(),
        query.category_id.as_deref(),
    )
    .await?;
    Ok(Json(expenses))
}

#[utoipa::path(
    post,
    path = "/api/v1/expenses",
    tag = "expenses",
    request_body = CreateExpense,
    responses(
        (status = 201, description = "Expense created", body = Expense),
        (status = 400, description = "Invalid amount, date, or category")
    )
)]
pub async fn create_expense(
    State(state): State<AppState>,
    Json(body): Json<CreateExpense>,
) -> Result<(StatusCode, Json<Expense>), AppError> {
    validate(&state, body.amount_cents, &body.spent_on, &body.category_id).await?;
    let expense = expenses::create(
        &state.pool,
        body.amount_cents,
        body.description.trim().to_owned(),
        body.category_id,
        body.spent_on,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(expense)))
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses/{id}",
    tag = "expenses",
    params(("id" = String, Path, description = "Expense id")),
    responses(
        (status = 200, description = "The expense", body = Expense),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn get_expense(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Expense>, AppError> {
    let expense = expenses::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(expense))
}

#[utoipa::path(
    put,
    path = "/api/v1/expenses/{id}",
    tag = "expenses",
    params(("id" = String, Path, description = "Expense id")),
    request_body = UpdateExpense,
    responses(
        (status = 200, description = "Updated expense", body = Expense),
        (status = 400, description = "Invalid amount, date, or category"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn update_expense(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateExpense>,
) -> Result<Json<Expense>, AppError> {
    validate(&state, body.amount_cents, &body.spent_on, &body.category_id).await?;
    let expense = expenses::update(
        &state.pool,
        &id,
        body.amount_cents,
        body.description.trim().to_owned(),
        body.category_id,
        body.spent_on,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(expense))
}

#[utoipa::path(
    delete,
    path = "/api/v1/expenses/{id}",
    tag = "expenses",
    params(("id" = String, Path, description = "Expense id")),
    responses(
        (status = 204, description = "Expense deleted"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn delete_expense(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !expenses::remove(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
