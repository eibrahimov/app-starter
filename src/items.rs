//! Example domain module: a minimal "items" resource wired through every
//! layer (migration -> queries -> API -> generated TS types -> UI page).
//! Replace it with your real domain; the wiring pattern stays the same.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItem {
    pub title: String,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Item>, sqlx::Error> {
    sqlx::query_as::<_, Item>(
        "SELECT id, title, done, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Item>, sqlx::Error> {
    sqlx::query_as::<_, Item>("SELECT id, title, done, created_at FROM items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, title: String) -> Result<Item, sqlx::Error> {
    let item = Item {
        id: Uuid::new_v4().to_string(),
        title,
        done: false,
        created_at: Utc::now(),
    };
    sqlx::query("INSERT INTO items (id, title, done, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(&item.id)
        .bind(&item.title)
        .bind(item.done)
        .bind(item.created_at)
        .execute(pool)
        .await?;
    Ok(item)
}

/// Returns true when a row was updated, false when the id does not exist.
pub async fn toggle(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE items SET done = NOT done WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Returns true when a row was deleted, false when the id does not exist.
pub async fn remove(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM items WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
