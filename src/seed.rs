//! OPTIONAL, DELETABLE seed data — a removable demo seam.
//!
//! Purpose: a fresh database is empty, so a first `cargo run` opens the UI on
//! empty states. Running with `--seed` (or `SEED=1`, or `just seed`) inserts a
//! handful of example rows so the worked examples — the post status lifecycle,
//! filters, pagination, the stats endpoint, and each plugin's resource — are
//! visible "in 30 seconds." It is OFF by default and never runs in tests, on a
//! normal boot, or in the Docker/compose defaults.
//!
//! Core example resources (posts) are seeded here directly; each registered
//! plugin contributes its own seed data through `Plugin::seed`, which this runner
//! iterates — so core never depends on a plugin.
//!
//! It is idempotent by design: each resource is seeded only when its table is
//! empty, so re-running (or leaving `--seed` on) never duplicates rows. The empty
//! check and the inserts are separate statements, so this assumes a single
//! seeding process. The core path goes through the same `posts` domain functions
//! the HTTP handlers use, exercising the real code instead of raw SQL.
//!
//! To remove the whole seam from a fork: delete this file and `tests/seed.rs`,
//! remove `pub mod seed;` from `src/lib.rs`, the `if args.seed … seed::run(…)`
//! block and `is_truthy` from `src/main.rs`, and the `seed` recipe from the
//! `justfile`.

use sqlx::SqlitePool;

use crate::posts::{self, PostStatus};

/// Seed example posts and every registered plugin's example rows, skipping any
/// resource that already has rows. Safe to call on every startup. Returns the
/// total number of rows inserted.
pub async fn run(pool: &SqlitePool) -> anyhow::Result<u64> {
    let posts_added = seed_posts(pool).await?;

    // Each plugin owns its seed data, so core never has to depend on a plugin.
    let mut plugins_added = 0;
    for plugin in crate::plugins::all() {
        plugins_added += plugin.seed(pool).await?;
    }

    let total = posts_added + plugins_added;
    if total == 0 {
        tracing::info!("seed: database already populated; nothing inserted");
    } else {
        tracing::info!(
            posts = posts_added,
            plugins = plugins_added,
            "seed: inserted example rows"
        );
    }
    Ok(total)
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
            "The todo plugin shows minimal CRUD; posts adds a draft -> published \
             -> archived lifecycle, filtered list queries with pagination, and a \
             stats endpoint.",
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

/// Total posts across all statuses. `posts::list` takes status/limit/offset
/// arguments, so the emptiness check reuses the existing `stats` aggregate
/// instead of fetching rows just to count them.
async fn post_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let stats = posts::stats(pool).await?;
    Ok(stats.draft + stats.published + stats.archived)
}
