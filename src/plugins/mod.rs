//! GENERATED registry of compiled-in plugins.
//!
//! The scaffolder (`just new-plugin`) inserts one `register()` line per plugin
//! into [`all`] -- the explicit link that forces the linker to include each
//! plugin crate (docs/plugin-framework.md §2). Empty until the first plugin is
//! migrated in (Phase 2).

use crate::Plugin;

/// Every registered plugin, in declaration order. The router, OpenAPI document,
/// and migration runner are all built by iterating this set.
pub fn all() -> Vec<Box<dyn Plugin>> {
    vec![
        todo::register(),
        // <scaffolder inserts register() calls here>
    ]
}
