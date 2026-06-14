//! HTTP handler for the dashboard summary: a month's total, per-category
//! breakdown, and a short month-over-month trend.

use crate::AppState;
use crate::error::AppError;
use crate::expenses::valid_month;
use crate::summary::{self, MonthSummary};
use axum::Json;
use axum::extract::{Query, State};
use chrono::Utc;
use serde::Deserialize;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SummaryQuery {
    /// Month to summarize, `YYYY-MM`. Defaults to the current UTC month.
    pub month: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/summary",
    tag = "summary",
    params(SummaryQuery),
    responses(
        (status = 200, description = "Spending summary for the month", body = MonthSummary),
        (status = 400, description = "Malformed month")
    )
)]
pub async fn get_summary(
    State(state): State<AppState>,
    Query(query): Query<SummaryQuery>,
) -> Result<Json<MonthSummary>, AppError> {
    let month = match query.month {
        Some(month) => {
            if !valid_month(&month) {
                return Err(AppError::BadRequest("month must be YYYY-MM".into()));
            }
            month
        }
        None => Utc::now().format("%Y-%m").to_string(),
    };
    Ok(Json(summary::month_summary(&state.pool, &month).await?))
}
