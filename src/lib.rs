pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod items;

use sqlx::SqlitePool;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
