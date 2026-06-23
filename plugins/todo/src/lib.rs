//! The `todo` plugin -- the first worked-example plugin.
//!
//! A minimal to-do resource (list/create/toggle/delete) wired through every
//! layer: migration (`todo_items`) -> queries -> handlers (`/api/v1/todo`) ->
//! OpenAPI + generated TS types -> the frontend page (`frontend/`). Copy this
//! shape for a new plugin (or run `just new-plugin`). Mirrors docs/plugin-framework.md §3.

use app_starter_plugin_api::{AppError, AppState, Plugin, SeedFuture};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

// ---- domain -------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
#[schema(as = todo_Todo)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = todo_CreateTodo)]
pub struct CreateTodo {
    pub title: String,
}

async fn list(pool: &SqlitePool) -> Result<Vec<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>(
        "SELECT id, title, done, created_at FROM todo_items ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Todo>, sqlx::Error> {
    sqlx::query_as::<_, Todo>("SELECT id, title, done, created_at FROM todo_items WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

async fn create(pool: &SqlitePool, title: String) -> Result<Todo, sqlx::Error> {
    let todo = Todo {
        id: Uuid::new_v4().to_string(),
        title,
        done: false,
        created_at: Utc::now(),
    };
    sqlx::query("INSERT INTO todo_items (id, title, done, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(&todo.id)
        .bind(&todo.title)
        .bind(todo.done)
        .bind(todo.created_at)
        .execute(pool)
        .await?;
    Ok(todo)
}

/// Returns true when a row was updated, false when the id does not exist.
async fn toggle(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE todo_items SET done = NOT done WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Returns true when a row was deleted, false when the id does not exist.
async fn remove(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM todo_items WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---- handlers -----------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/v1/todo",
    tag = "todo",
    responses((status = 200, description = "All to-dos, newest first", body = [Todo]))
)]
async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    Ok(Json(list(&state.pool).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/todo",
    tag = "todo",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "To-do created", body = Todo),
        (status = 400, description = "Empty title")
    )
)]
async fn create_todo(
    State(state): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = body.title.trim().to_owned();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".into()));
    }
    let todo = create(&state.pool, title).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

#[utoipa::path(
    post,
    path = "/api/v1/todo/{id}/toggle",
    tag = "todo",
    params(("id" = String, Path, description = "To-do id")),
    responses(
        (status = 200, description = "Toggled to-do", body = Todo),
        (status = 404, description = "Unknown id")
    )
)]
async fn toggle_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    if !toggle(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let todo = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(todo))
}

#[utoipa::path(
    delete,
    path = "/api/v1/todo/{id}",
    tag = "todo",
    params(("id" = String, Path, description = "To-do id")),
    responses(
        (status = 204, description = "To-do deleted"),
        (status = 404, description = "Unknown id")
    )
)]
async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !remove(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---- seed ---------------------------------------------------------------

/// Inserts a few to-dos (a mix of done/not-done) when the table is empty. Called
/// by the host's seed runner via [`Plugin::seed`].
async fn seed_todos(pool: &SqlitePool) -> anyhow::Result<u64> {
    if !list(pool).await?.is_empty() {
        return Ok(0);
    }

    // (title, done)
    let fixtures = [
        ("Read AGENTS.md for the project conventions", true),
        ("Explore the todo and posts worked examples", true),
        ("Add your first plugin with the add-plugin skill", false),
        ("Replace todo and posts with your own domain", false),
    ];

    let mut inserted = 0;
    for (title, done) in fixtures {
        let todo = create(pool, title.to_owned()).await?;
        if done {
            toggle(pool, &todo.id).await?;
        }
        inserted += 1;
    }
    Ok(inserted)
}

// ---- plugin -------------------------------------------------------------

struct TodoPlugin;

impl Plugin for TodoPlugin {
    fn name(&self) -> &'static str {
        "todo"
    }

    fn host_api(&self) -> &'static str {
        "^1"
    }

    fn api(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
            .routes(routes!(list_todos, create_todo))
            .routes(routes!(toggle_todo))
            .routes(routes!(delete_todo))
    }

    fn migrator(&self) -> Option<Migrator> {
        Some(sqlx::migrate!("./migrations"))
    }

    fn seed<'a>(&'a self, pool: &'a SqlitePool) -> SeedFuture<'a> {
        Box::pin(async move { seed_todos(pool).await })
    }
}

/// Registration hook the host's generated `src/plugins.rs` calls -- the
/// explicit link that makes the linker include this crate.
pub fn register() -> Box<dyn Plugin> {
    Box::new(TodoPlugin)
}
