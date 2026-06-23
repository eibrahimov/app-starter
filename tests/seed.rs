//! Tests for the optional seed seam (`src/seed.rs`). Part of the removable
//! seam: delete this file along with `src/seed.rs` when dropping seed support.
//!
//! These call `seed::run` explicitly against an isolated in-memory database;
//! the seam never seeds on its own during the normal `just test` boot.

use app_starter::{posts, seed};
use sqlx::SqlitePool;

mod common;

/// Rows in the `todo` plugin's table. The plugin's domain functions are private,
/// so the seed test verifies the plugin contributed rows by counting its table
/// directly (the table name is part of the plugin's public schema contract).
async fn todo_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM todo_items")
        .fetch_one(pool)
        .await
        .expect("count todo_items")
}

#[tokio::test]
async fn seed_populates_an_empty_database() {
    let pool = common::memory_pool().await;

    let inserted = seed::run(&pool).await.expect("seed runs");
    assert!(inserted > 0, "seeding an empty database should insert rows");

    // Plugin seed ran (the todo plugin contributes its rows via Plugin::seed).
    assert!(
        todo_count(&pool).await > 0,
        "the todo plugin should be seeded"
    );

    // Posts cover the full lifecycle so every status filter has data to show.
    let stats = posts::stats(&pool).await.unwrap();
    assert!(stats.draft > 0, "expected at least one draft post");
    assert!(stats.published > 0, "expected at least one published post");
    assert!(stats.archived > 0, "expected at least one archived post");
}

#[tokio::test]
async fn seed_is_idempotent() {
    let pool = common::memory_pool().await;

    let first = seed::run(&pool).await.expect("first seed");
    assert!(first > 0, "first run should insert rows");

    let todo_before = todo_count(&pool).await;
    let before = posts::stats(&pool).await.unwrap();
    let post_total = before.draft + before.published + before.archived;

    // A second run must insert nothing and leave the data untouched.
    let second = seed::run(&pool).await.expect("second seed");
    assert_eq!(second, 0, "re-running seed must not insert more rows");

    assert_eq!(todo_count(&pool).await, todo_before);
    let after = posts::stats(&pool).await.unwrap();
    assert_eq!(after.draft + after.published + after.archived, post_total);
}
