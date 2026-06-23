//! Shared helpers for the integration test crates, included with `mod common;`
//! from `tests/api.rs` and `tests/seed.rs`.
//!
//! Kept as a flat `tests/common.rs` (this codebase has no `mod.rs` files).
//! Cargo also compiles every top-level `tests/*.rs` as its own test binary, so
//! this file builds standalone too — where `memory_pool` is unused. The
//! file-level allow below silences that `dead_code` warning under
//! `clippy --all-targets -- -D warnings`.
#![allow(dead_code)]

use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

/// An isolated in-memory SQLite pool with the migrations applied.
///
/// `max_connections(1)` is required, not a tuning choice: every `sqlite::memory:`
/// connection is its own private database, so a larger pool would hand back
/// connections that never saw the migrated schema.
pub async fn memory_pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    app_starter::db::run_all_migrators(&pool)
        .await
        .expect("run migrations");
    pool
}
