//! Tests for the optional seed seam (`src/seed.rs`). Part of the removable
//! seam: delete this file along with `src/seed.rs` when dropping seed support.
//!
//! These call `seed::run` explicitly against an isolated in-memory database;
//! the seam never seeds on its own during the normal `just test` boot.

use app_starter::seed;
use sqlx::SqlitePool;

mod common;

// The worked-example plugins' domains are private to their crates, so the seed
// test verifies each contributed rows by counting its table directly (the table
// name is part of the plugin's public schema contract).
async fn todo_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM todo_items")
        .fetch_one(pool)
        .await
        .expect("count todo_items")
}

async fn blog_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blog_posts")
        .fetch_one(pool)
        .await
        .expect("count blog_posts")
}

#[tokio::test]
async fn seed_populates_an_empty_database() {
    let pool = common::memory_pool().await;

    let inserted = seed::run(&pool).await.expect("seed runs");
    assert!(inserted > 0, "seeding an empty database should insert rows");

    // Both worked-example plugins contribute their rows via Plugin::seed.
    assert!(
        todo_count(&pool).await > 0,
        "the todo plugin should be seeded"
    );
    assert!(
        blog_count(&pool).await > 0,
        "the blog plugin should be seeded"
    );
}

#[tokio::test]
async fn seed_is_idempotent() {
    let pool = common::memory_pool().await;

    let first = seed::run(&pool).await.expect("first seed");
    assert!(first > 0, "first run should insert rows");

    let todo_before = todo_count(&pool).await;
    let blog_before = blog_count(&pool).await;

    // A second run must insert nothing and leave the data untouched.
    let second = seed::run(&pool).await.expect("second seed");
    assert_eq!(second, 0, "re-running seed must not insert more rows");

    assert_eq!(todo_count(&pool).await, todo_before);
    assert_eq!(blog_count(&pool).await, blog_before);
}
