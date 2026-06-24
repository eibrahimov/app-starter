//! OPTIONAL, DELETABLE seed data — a removable demo seam.
//!
//! Purpose: a fresh database is empty, so a first `cargo run` opens the UI on
//! empty states. Running with `--seed` (or `SEED=1`, or `just seed`) inserts a
//! handful of example rows so the worked examples are visible "in 30 seconds."
//! It is OFF by default and never runs in tests, on a normal boot, or in the
//! Docker/compose defaults.
//!
//! Core has no example resources of its own anymore — each are plugins — so this
//! runner simply iterates every registered plugin's `Plugin::seed` (core never
//! depends on a plugin). It is idempotent by design: each plugin seeds only when
//! its table is empty, so re-running never duplicates rows.
//!
//! To remove the whole seam from a fork: delete this file and `tests/seed.rs`,
//! remove `pub mod seed;` from `src/lib.rs`, the `if args.seed … seed::run(…)`
//! block and `is_truthy` from `src/main.rs`, and the `seed` recipe from the
//! `justfile`.

use sqlx::SqlitePool;

/// Seed every registered plugin's example rows, skipping any plugin whose table
/// already has rows. Safe to call on every startup. Returns total rows inserted.
pub async fn run(pool: &SqlitePool) -> anyhow::Result<u64> {
    let mut total = 0;
    for plugin in crate::plugins::all() {
        total += plugin.seed(pool).await?;
    }

    if total == 0 {
        tracing::info!("seed: database already populated; nothing inserted");
    } else {
        tracing::info!(total, "seed: inserted example rows");
    }
    Ok(total)
}
