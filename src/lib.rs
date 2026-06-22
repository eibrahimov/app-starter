pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod items;
pub mod plugin;
pub mod plugins;
pub mod posts;
pub mod seed;

/// The plugin contract, re-exported at the crate root so plugin crates can
/// write `impl app_starter::Plugin`.
pub use plugin::{PLUGIN_API_VERSION, Plugin};

use sqlx::SqlitePool;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}
