//! Expenses domain: the core record of the tracker. Amounts are integer minor
//! units (cents). Each expense optionally belongs to a category; the category
//! name is joined in for convenient display and CSV export. Pure date/month
//! validators live here and are unit-tested below.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Expense {
    pub id: String,
    /// Amount in integer minor units (cents). Always positive.
    pub amount_cents: i64,
    pub description: String,
    pub category_id: Option<String>,
    /// Denormalized category name (via LEFT JOIN) for display; `None` when
    /// uncategorized or the category was deleted.
    pub category_name: Option<String>,
    /// Calendar date the money was spent, `YYYY-MM-DD`.
    pub spent_on: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateExpense {
    pub amount_cents: i64,
    #[serde(default)]
    pub description: String,
    pub category_id: Option<String>,
    pub spent_on: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateExpense {
    pub amount_cents: i64,
    #[serde(default)]
    pub description: String,
    pub category_id: Option<String>,
    pub spent_on: String,
}

/// True when `value` is a valid `YYYY-MM-DD` calendar date.
pub fn valid_date(value: &str) -> bool {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

/// True when `value` is a valid `YYYY-MM` month (uses day 01 to validate).
pub fn valid_month(value: &str) -> bool {
    NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d").is_ok()
}

const SELECT_COLUMNS: &str = "SELECT e.id, e.amount_cents, e.description, e.category_id, \
     c.name AS category_name, e.spent_on, e.created_at \
     FROM expenses e LEFT JOIN categories c ON c.id = e.category_id";

/// Lists expenses, optionally filtered by `YYYY-MM` month and/or category id,
/// newest first.
pub async fn list(
    pool: &SqlitePool,
    month: Option<&str>,
    category_id: Option<&str>,
) -> Result<Vec<Expense>, sqlx::Error> {
    let mut sql = String::from(SELECT_COLUMNS);
    let mut clauses = Vec::new();
    if month.is_some() {
        clauses.push("e.spent_on LIKE ?1");
    }
    if category_id.is_some() {
        // Numbered binds are positional; compute the next index from what we add.
        clauses.push(if month.is_some() {
            "e.category_id = ?2"
        } else {
            "e.category_id = ?1"
        });
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY e.spent_on DESC, e.created_at DESC");

    let mut query = sqlx::query_as::<_, Expense>(&sql);
    if let Some(month) = month {
        query = query.bind(format!("{month}%"));
    }
    if let Some(category_id) = category_id {
        query = query.bind(category_id.to_owned());
    }
    query.fetch_all(pool).await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Expense>, sqlx::Error> {
    sqlx::query_as::<_, Expense>(&format!("{SELECT_COLUMNS} WHERE e.id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &SqlitePool,
    amount_cents: i64,
    description: String,
    category_id: Option<String>,
    spent_on: String,
) -> Result<Expense, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO expenses (id, amount_cents, description, category_id, spent_on, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(amount_cents)
    .bind(&description)
    .bind(&category_id)
    .bind(&spent_on)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    // Re-read so the response carries the joined category name consistently.
    get(pool, &id).await.map(|e| e.expect("just inserted"))
}

/// Returns the updated expense, or `None` when the id does not exist.
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    amount_cents: i64,
    description: String,
    category_id: Option<String>,
    spent_on: String,
) -> Result<Option<Expense>, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE expenses SET amount_cents = ?2, description = ?3, category_id = ?4, \
         spent_on = ?5 WHERE id = ?1",
    )
    .bind(id)
    .bind(amount_cents)
    .bind(description)
    .bind(category_id)
    .bind(spent_on)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Ok(None);
    }
    get(pool, id).await
}

/// Returns true when a row was deleted, false when the id does not exist.
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM expenses WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_date_accepts_iso_dates_and_rejects_junk() {
        assert!(valid_date("2026-06-14"));
        assert!(valid_date("2000-01-01"));
        assert!(!valid_date("2026-13-01"));
        assert!(!valid_date("2026-06"));
        assert!(!valid_date("14/06/2026"));
        assert!(!valid_date(""));
    }

    #[test]
    fn valid_month_accepts_year_month_and_rejects_junk() {
        assert!(valid_month("2026-06"));
        assert!(valid_month("1999-12"));
        assert!(!valid_month("2026-13"));
        assert!(!valid_month("2026-06-14"));
        assert!(!valid_month("2026"));
        assert!(!valid_month(""));
    }
}
