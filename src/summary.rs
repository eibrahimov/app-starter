//! Summary domain: read-only aggregates that power the dashboard — a month's
//! total spend, a per-category breakdown (with budgets for budget-vs-actual),
//! and a short month-over-month trend. All amounts are integer minor units.

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CategorySpend {
    /// `None` for the synthetic "Uncategorized" bucket.
    pub category_id: Option<String>,
    pub name: String,
    pub color: String,
    pub spent_cents: i64,
    /// Monthly budget in cents, when the category sets one.
    pub budget_cents: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthTotal {
    /// `YYYY-MM`.
    pub month: String,
    pub total_cents: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthSummary {
    /// The month this summary covers, `YYYY-MM`.
    pub month: String,
    pub total_cents: i64,
    /// Per-category spend for the month: every category that was spent on or
    /// that sets a budget, plus an "Uncategorized" bucket when relevant.
    pub categories: Vec<CategorySpend>,
    /// Totals for up to the last 6 months ending at `month`, oldest first.
    pub recent_months: Vec<MonthTotal>,
}

/// Number of trailing months (including the selected one) in the trend.
const TREND_MONTHS: i64 = 6;

pub async fn month_summary(pool: &SqlitePool, month: &str) -> Result<MonthSummary, sqlx::Error> {
    let like = format!("{month}%");

    let total_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM expenses WHERE spent_on LIKE ?1",
    )
    .bind(&like)
    .fetch_one(pool)
    .await?;

    // Every category with its spend for the month; keep those with spend or a budget.
    let category_rows: Vec<(String, String, String, Option<i64>, i64)> = sqlx::query_as(
        "SELECT c.id, c.name, c.color, c.monthly_budget_cents, \
         COALESCE((SELECT SUM(e.amount_cents) FROM expenses e \
                   WHERE e.category_id = c.id AND e.spent_on LIKE ?1), 0) AS spent \
         FROM categories c \
         ORDER BY spent DESC, c.name COLLATE NOCASE ASC",
    )
    .bind(&like)
    .fetch_all(pool)
    .await?;

    let mut categories: Vec<CategorySpend> = category_rows
        .into_iter()
        .filter(|(_, _, _, budget, spent)| *spent > 0 || budget.is_some())
        .map(|(id, name, color, budget, spent)| CategorySpend {
            category_id: Some(id),
            name,
            color,
            spent_cents: spent,
            budget_cents: budget,
        })
        .collect();

    // Synthetic bucket for expenses with no category.
    let uncategorized: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0) FROM expenses \
         WHERE category_id IS NULL AND spent_on LIKE ?1",
    )
    .bind(&like)
    .fetch_one(pool)
    .await?;
    if uncategorized > 0 {
        categories.push(CategorySpend {
            category_id: None,
            name: "Uncategorized".to_owned(),
            color: "#52525b".to_owned(),
            spent_cents: uncategorized,
            budget_cents: None,
        });
    }
    categories.sort_by_key(|c| std::cmp::Reverse(c.spent_cents));

    // Month-over-month totals up to and including `month`, newest first then
    // reversed to oldest-first for left-to-right charting.
    let mut recent_months: Vec<MonthTotal> = sqlx::query_as::<_, (String, i64)>(
        "SELECT substr(spent_on, 1, 7) AS m, SUM(amount_cents) \
         FROM expenses WHERE substr(spent_on, 1, 7) <= ?1 \
         GROUP BY m ORDER BY m DESC LIMIT ?2",
    )
    .bind(month)
    .bind(TREND_MONTHS)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(month, total_cents)| MonthTotal { month, total_cents })
    .collect();
    recent_months.reverse();

    Ok(MonthSummary {
        month: month.to_owned(),
        total_cents,
        categories,
        recent_months,
    })
}
