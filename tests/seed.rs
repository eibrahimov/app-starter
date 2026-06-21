//! Tests for the optional seed seam (`src/seed.rs`). Part of the removable
//! seam: delete this file along with `src/seed.rs` when dropping seed support.
//!
//! These call `seed::run` explicitly against an isolated in-memory database;
//! the seam never seeds on its own during the normal `just test` boot.

use app_starter::{items, posts, seed};

mod common;

#[tokio::test]
async fn seed_populates_an_empty_database() {
    let pool = common::memory_pool().await;

    let inserted = seed::run(&pool).await.expect("seed runs");
    assert!(inserted > 0, "seeding an empty database should insert rows");

    assert!(
        !items::list(&pool).await.unwrap().is_empty(),
        "items should be populated"
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

    let item_count = items::list(&pool).await.unwrap().len();
    let before = posts::stats(&pool).await.unwrap();
    let post_total = before.draft + before.published + before.archived;

    // A second run must insert nothing and leave the data untouched.
    let second = seed::run(&pool).await.expect("second seed");
    assert_eq!(second, 0, "re-running seed must not insert more rows");

    assert_eq!(items::list(&pool).await.unwrap().len(), item_count);
    let after = posts::stats(&pool).await.unwrap();
    assert_eq!(after.draft + after.published + after.archived, post_total);
}
