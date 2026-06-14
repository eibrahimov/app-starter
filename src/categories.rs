//! Categories domain: a labelled, colored bucket an expense can belong to,
//! with an optional monthly budget. Mirrors the plain-function query style of
//! the rest of the app (no repository structs/traits).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: String,
    /// Optional monthly budget in integer minor units (cents). `None` = no budget.
    pub monthly_budget_cents: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCategory {
    pub name: String,
    /// Hex color like `#6366f1`. Defaults to a neutral indigo when omitted.
    #[serde(default = "default_color")]
    pub color: String,
    pub monthly_budget_cents: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCategory {
    pub name: String,
    pub color: String,
    pub monthly_budget_cents: Option<i64>,
}

fn default_color() -> String {
    "#6366f1".to_owned()
}

const SELECT_COLUMNS: &str =
    "SELECT id, name, color, monthly_budget_cents, created_at FROM categories";

pub async fn list(pool: &SqlitePool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(&format!(
        "{SELECT_COLUMNS} ORDER BY name COLLATE NOCASE ASC"
    ))
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(&format!("{SELECT_COLUMNS} WHERE id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &SqlitePool,
    name: String,
    color: String,
    monthly_budget_cents: Option<i64>,
) -> Result<Category, sqlx::Error> {
    let category = Category {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        monthly_budget_cents,
        created_at: Utc::now(),
    };
    sqlx::query(
        "INSERT INTO categories (id, name, color, monthly_budget_cents, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&category.id)
    .bind(&category.name)
    .bind(&category.color)
    .bind(category.monthly_budget_cents)
    .bind(category.created_at)
    .execute(pool)
    .await?;
    Ok(category)
}

/// Returns true when a row was updated, false when the id does not exist.
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    name: String,
    color: String,
    monthly_budget_cents: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE categories SET name = ?2, color = ?3, monthly_budget_cents = ?4 WHERE id = ?1",
    )
    .bind(id)
    .bind(name)
    .bind(color)
    .bind(monthly_budget_cents)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes a category and uncategorizes its expenses in one transaction.
/// SQLite has no foreign-key cascade enabled by default, so we null the
/// referencing rows explicitly. Returns false when the id does not exist.
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE expenses SET category_id = NULL WHERE category_id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let result = sqlx::query("DELETE FROM categories WHERE id = ?1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}
