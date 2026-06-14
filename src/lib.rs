pub mod api;
pub mod categories;
pub mod db;
pub mod error;
pub mod expenses;
pub mod frontend;
pub mod settings;
pub mod summary;

use sqlx::SqlitePool;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
