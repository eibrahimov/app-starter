//! The host<->plugin contract crate.
//!
//! Holds the `Plugin` trait, `AppState`, `AppError`, and `PLUGIN_API_VERSION` in
//! a leaf crate that BOTH the host (`app-starter`) and every plugin crate depend
//! on. This is what breaks the dependency cycle: the host's generated registry
//! (`app-starter`'s `src/plugins.rs`) references each plugin crate, and each
//! plugin references this crate -- never `app-starter` -- so the package graph
//! stays acyclic. The host re-exports these items, so `app_starter::{AppState,
//! Plugin, PLUGIN_API_VERSION}` and `app_starter::error::AppError` keep resolving.
//!
//! This crate must never depend on `app-starter` (see
//! docs/plugin-framework-impl-status.md, iter-4 blocker).

use std::future::Future;
use std::pin::Pin;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use utoipa_axum::router::OpenApiRouter;

/// Shared application state, cloned into every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

/// Application error type. Every handler (core and plugin) returns
/// `Result<_, AppError>`, mapped onto an HTTP status + JSON body here. It lives in
/// the contract crate so plugin handlers can return it without depending on the
/// host.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Sqlx(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
            AppError::Other(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Host plugin-API version. A plugin's [`Plugin::host_api`] is a semver range
/// checked against this constant when the registry is assembled; an out-of-range
/// plugin is refused with a human-readable error (docs/plugin-framework.md §3).
pub const PLUGIN_API_VERSION: &str = "1.0.0";

/// The boxed, `Send` future returned by [`Plugin::seed`]. Spelled out (rather
/// than `async fn`) so the trait stays object-safe for `Box<dyn Plugin>`.
pub type SeedFuture<'a> = Pin<Box<dyn Future<Output = anyhow::Result<u64>> + Send + 'a>>;

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

    /// Required host-API semver range, checked against [`PLUGIN_API_VERSION`] at
    /// startup by the host (`db::validate_registry`). The plugin's `plugin.toml`
    /// `host_api` is an informational mirror of this value, not the enforced
    /// source — this method is what the host actually reads.
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

    /// Optional example seed data, inserted by the host's seed runner. Implement
    /// idempotently (skip when the plugin's table is already populated) and return
    /// the number of rows inserted. Default: no-op. Keeps `core never depends on a
    /// plugin` intact -- the host iterates plugins instead of importing them.
    fn seed<'a>(&'a self, pool: &'a SqlitePool) -> SeedFuture<'a> {
        let _ = pool;
        Box::pin(async { Ok(0) })
    }
}
