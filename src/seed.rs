//! OPTIONAL, DELETABLE seed data — a removable demo seam.
//!
//! Purpose: a fresh database is empty, so a first `cargo run` opens the UI on
//! empty states for both Items and Posts. Running with `--seed` (or `SEED=1`,
//! or `just seed`) inserts a handful of example rows so the worked examples —
//! the post status lifecycle, filters, pagination, and the stats endpoint —
//! are visible "in 30 seconds." It is OFF by default and never runs in tests,
//! on a normal boot, or in the Docker/compose defaults.
//!
//! This is a worked example to copy or delete, not core infrastructure. To
//! remove the whole seam from a fork:
//!   1. delete this file (`src/seed.rs`) and its test (`tests/seed.rs`),
//!   2. remove `pub mod seed;` from `src/lib.rs`,
//!   3. remove the `seed` field, the `if args.seed ... seed::run(...)` block,
//!      and the `is_truthy` helper from `src/main.rs`,
//!   4. delete the `seed` recipe from the `justfile`.
//!
//! Nothing else depends on it.
//!
//! It is idempotent by design: each resource is seeded only when its table is
//! empty, so re-running (or leaving `--seed` on) never duplicates rows and
//! never overwrites real data. The empty check and the inserts are separate
//! statements, so this assumes a single seeding process — two instances
//! booting with `--seed` against the same database could each seed once. That
//! is fine for the single-instance startup this seam targets. It deliberately
//! goes through the same `items`/`posts` domain functions the HTTP handlers
//! use, so the seed path exercises the real code instead of bypassing it with
//! raw SQL.

use sqlx::SqlitePool;

use crate::items;
use crate::posts::{self, PostStatus};

/// Seed example items and posts, skipping any resource that already has rows.
/// Safe to call on every startup. Returns the total number of rows inserted.
pub async fn run(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let items_added = seed_items(pool).await?;
    let posts_added = seed_posts(pool).await?;
    let total = items_added + posts_added;
    if total == 0 {
        tracing::info!("seed: database already populated; nothing inserted");
    } else {
        tracing::info!(
            items = items_added,
            posts = posts_added,
            "seed: inserted example rows"
        );
    }
    Ok(total)
}

/// Inserts a few items (a mix of done and not-done) when the table is empty.
async fn seed_items(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    if !items::list(pool).await?.is_empty() {
        return Ok(0);
    }

    // (title, done)
    let fixtures = [
        ("Read AGENTS.md for the project conventions", true),
        ("Explore the items and posts worked examples", true),
        ("Add your first resource with the add-resource skill", false),
        ("Replace items and posts with your own domain", false),
    ];

    let mut inserted = 0;
    for (title, done) in fixtures {
        let item = items::create(pool, title.to_owned()).await?;
        if done {
            items::toggle(pool, &item.id).await?;
        }
        inserted += 1;
    }
    Ok(inserted)
}

/// Inserts a mix of draft/published/archived posts when the table is empty,
/// driving each row through the real lifecycle transitions (`create` ->
/// `publish` -> `archive`) so the seed exercises the same code the API does.
async fn seed_posts(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    if post_count(pool).await? > 0 {
        return Ok(0);
    }

    // (title, body, target status)
    let fixtures = [
        (
            "Welcome to App Starter",
            "This post was created by the optional --seed routine. It is example \
             data — delete src/seed.rs to remove the seam.",
            PostStatus::Published,
        ),
        (
            "How the worked examples fit together",
            "Items shows minimal CRUD; posts adds a draft -> published -> archived \
             lifecycle, filtered list queries with pagination, and a stats endpoint.",
            PostStatus::Published,
        ),
        (
            "A work-in-progress draft",
            "Drafts stay unpublished until you publish them. This row demonstrates \
             the draft state and the status filter.",
            PostStatus::Draft,
        ),
        (
            "An archived announcement",
            "Archived posts are retained but kept out of the active flow. This row \
             demonstrates the archived state.",
            PostStatus::Archived,
        ),
    ];

    let mut inserted = 0;
    for (title, body, target) in fixtures {
        let post = posts::create(pool, title.to_owned(), body.to_owned()).await?;
        match target {
            PostStatus::Draft => {}
            PostStatus::Published => {
                posts::publish(pool, &post.id).await?;
            }
            PostStatus::Archived => {
                posts::publish(pool, &post.id).await?;
                posts::archive(pool, &post.id).await?;
            }
        }
        inserted += 1;
    }
    Ok(inserted)
}

/// Total posts across all statuses. Unlike `items::list`, `posts::list` takes
/// status/limit/offset arguments, so the emptiness check reuses the existing
/// `stats` aggregate instead of fetching rows just to count them.
async fn post_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let stats = posts::stats(pool).await?;
    Ok(stats.draft + stats.published + stats.archived)
}
