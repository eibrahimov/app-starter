//! Guard tests for the plugin framework (docs/plugin-framework.md §6). They
//! assert the namespacing invariants hold across every registered plugin, so a
//! new plugin that violates them fails CI instead of silently colliding.

use std::collections::{HashMap, HashSet};

use app_starter::AppState;

mod common;

/// Every plugin's routes live under its derived prefix `/api/v1/<name>`.
#[test]
fn every_plugin_route_is_under_its_derived_prefix() {
    for plugin in app_starter::plugins::all() {
        let name = plugin.name();
        let prefix = format!("/api/v1/{name}");
        let (_router, api): (axum::Router<AppState>, _) = plugin.api().split_for_parts();
        for path in api.paths.paths.keys() {
            assert!(
                path.starts_with(&prefix),
                "plugin '{name}' serves route '{path}' outside its derived prefix '{prefix}'"
            );
        }
    }
}

/// Plugin OpenAPI component names are prefixed by the plugin name, and no two
/// plugins define the same component name. Prefixing makes the silent
/// last-wins-overwrite of same-named components impossible by construction; this
/// guards the derivation (a plugin that forgets to prefix fails here).
#[test]
fn no_cross_plugin_schema_name_collisions() {
    let mut owner: HashMap<String, &'static str> = HashMap::new();
    for plugin in app_starter::plugins::all() {
        let name = plugin.name();
        let prefix = format!("{name}_");
        let (_router, api): (axum::Router<AppState>, _) = plugin.api().split_for_parts();
        let Some(components) = api.components else {
            continue;
        };
        for schema_name in components.schemas.keys() {
            assert!(
                schema_name.starts_with(&prefix),
                "plugin '{name}' component '{schema_name}' is not prefixed with '{prefix}'"
            );
            if let Some(other) = owner.insert(schema_name.clone(), name) {
                panic!("component '{schema_name}' is defined by both '{other}' and '{name}'");
            }
        }
    }
}

/// Every plugin-created table is prefixed with its plugin name, and plugin names
/// (hence the `_sqlx_migrations_<name>` keyspaces) are unique.
#[tokio::test]
async fn plugin_tables_are_prefixed_and_unique() {
    let names: Vec<&'static str> = app_starter::plugins::all()
        .iter()
        .map(|p| p.name())
        .collect();
    let unique: HashSet<&'static str> = names.iter().copied().collect();
    assert_eq!(
        names.len(),
        unique.len(),
        "plugin names must be unique: {names:?}"
    );

    // memory_pool() runs run_all_migrators, so every plugin's tables now exist.
    let pool = common::memory_pool().await;
    let prefixes: Vec<String> = names.iter().map(|n| format!("{n}_")).collect();
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(&pool)
            .await
            .expect("list tables");

    for table in &tables {
        // Skip sqlite internals and the migration tracking tables (core +
        // per-plugin); the rest are application tables and must be plugin-prefixed.
        if table.starts_with("sqlite_") || table.starts_with("_sqlx_migrations") {
            continue;
        }
        assert!(
            prefixes.iter().any(|p| table.starts_with(p)),
            "table '{table}' is not prefixed by any registered plugin name {names:?}"
        );
    }
}

/// The expected plugins are actually registered. The silent-registry-loss risk is
/// a release `lto`+`strip` phenomenon, so CI ALSO runs a release-profile smoke
/// check on the served `/api/openapi.json` (`just release-smoke`); this debug test
/// is the fast-feedback counterpart.
#[test]
fn expected_plugins_are_registered() {
    let names: HashSet<&'static str> = app_starter::plugins::all()
        .iter()
        .map(|p| p.name())
        .collect();
    for expected in ["todo", "blog"] {
        assert!(
            names.contains(expected),
            "expected plugin '{expected}' is not registered; got {names:?}"
        );
    }
}
