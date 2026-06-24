//! `AppError` now lives in the `app-starter-plugin-api` contract crate, so plugin
//! handlers can return it without depending on the host. Re-exported here so
//! `crate::error::AppError` keeps resolving across the codebase.
pub use app_starter_plugin_api::AppError;
