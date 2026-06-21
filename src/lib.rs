pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod items;
pub mod posts;
pub mod seed;

use sqlx::SqlitePool;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
