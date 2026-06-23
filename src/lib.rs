pub mod api;
pub mod db;
pub mod error;
pub mod frontend;
pub mod plugins;
pub mod seed;

/// The plugin contract + shared state live in the leaf `app-starter-plugin-api`
/// crate, which plugins also depend on -- this breaks the host<->plugin cycle a
/// generated registry would otherwise create. Re-exported here so existing
/// `app_starter::{AppState, Plugin, PLUGIN_API_VERSION}` paths keep resolving.
pub use app_starter_plugin_api::{AppState, PLUGIN_API_VERSION, Plugin};
