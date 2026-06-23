//! The host<->plugin contract crate.
//!
//! Holds the `Plugin` trait, `AppState`, and `PLUGIN_API_VERSION` in a leaf crate
//! that BOTH the host (`app-starter`) and every plugin crate depend on. This is
//! what breaks the dependency cycle: the host's generated registry
//! (`app-starter`'s `src/plugins/mod.rs`) references each plugin crate, and each
//! plugin references this crate -- never `app-starter` -- so the package graph
//! stays acyclic. The host re-exports these items, so
//! `app_starter::{AppState, Plugin, PLUGIN_API_VERSION}` keep resolving.
//!
//! This crate must never depend on `app-starter` (see
//! docs/plugin-framework-impl-status.md, iter-4 blocker).

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use utoipa_axum::router::OpenApiRouter;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

/// Host plugin-API version. A plugin's [`Plugin::host_api`] is a semver range
/// checked against this constant when the registry is assembled; an out-of-range
/// plugin is refused with a human-readable error (docs/plugin-framework.md §3).
pub const PLUGIN_API_VERSION: &str = "1.0.0";

/// The contract every plugin implements.
///
/// Object-safe (every method takes `&self` and returns a `Self`-free type) so
/// the registry can hold `Box<dyn Plugin>`.
pub trait Plugin: Send + Sync + 'static {
    /// Stable identifier; the namespace key from which the route prefix
    /// (`/api/v1/<name>`), OpenAPI component prefix (`<name>_*`), and table
    /// prefix (`<name>_*`) all derive. One source of truth, not a separately
    /// declared prefix.
    fn name(&self) -> &'static str;

    /// Required host-API semver range (from the plugin's `plugin.toml`
    /// `host_api`), checked against [`PLUGIN_API_VERSION`].
    fn host_api(&self) -> &'static str;

    /// Routes AND their OpenAPI paths/schemas, built together so they cannot
    /// desync. `utoipa-axum`'s [`OpenApiRouter`] registers a handler once and
    /// contributes both the axum route and the OpenAPI path + schemas.
    fn api(&self) -> OpenApiRouter<AppState>;

    /// Migrations this plugin owns, embedded at compile time (plugin-relative).
    ///
    /// The host -- not the plugin -- applies a per-plugin tracking-table name
    /// (`_sqlx_migrations_<name>`) before running it, so a plugin simply returns
    /// its bare `sqlx::migrate!("./migrations")` Migrator.
    fn migrator(&self) -> Option<Migrator> {
        None
    }
}
