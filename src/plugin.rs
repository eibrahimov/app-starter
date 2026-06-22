//! The plugin contract.
//!
//! A plugin is a Cargo workspace-member crate that contributes routes together
//! with their OpenAPI fragment (built from one declaration so they cannot drift)
//! and, optionally, its own migrations. Registration is **explicit**: the
//! generated [`crate::plugins`] module names each plugin's `register()` so the
//! linker is forced to include the crate. See docs/plugin-framework.md §2-3.

use crate::AppState;
use sqlx::migrate::Migrator;
use utoipa_axum::router::OpenApiRouter;

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
