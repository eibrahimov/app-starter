use crate::{PLUGIN_API_VERSION, Plugin};
use anyhow::Context;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;

/// Connect to SQLite and run pending migrations.
///
/// Creates the parent directory of a file-backed database so first run
/// works out of the box with the default `sqlite://data/app.db?mode=rwc`.
pub async fn init(database_url: &str) -> anyhow::Result<SqlitePool> {
    if let Some(rest) = database_url.strip_prefix("sqlite://") {
        let path = rest.split('?').next().unwrap_or(rest);
        if path != ":memory:"
            && !path.is_empty()
            && let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    run_all_migrators(&pool).await?;
    Ok(pool)
}

/// Runs the core migrations, then each registered plugin's migrations into its
/// own `_sqlx_migrations_<name>` tracking table, so every plugin owns an
/// independent version keyspace (docs/plugin-framework.md §5).
///
/// Sets `busy_timeout` + WAL on the pool first so a file-backed database
/// tolerates brief contention during startup. Both [`init`] and the integration
/// tests route through here, so the test schema always matches `init`.
pub async fn run_all_migrators(pool: &SqlitePool) -> anyhow::Result<()> {
    // Assemble the registry once and refuse an incompatible or malformed plugin
    // BEFORE touching the database, so a bad build fails the boot with a clear
    // error instead of half-migrating (docs/plugin-framework.md §3).
    let plugins = crate::plugins::all();
    validate_registry(&plugins)?;

    // A small busy timeout + WAL keep startup resilient to brief contention on a
    // file-backed DB (§5 M4). On `sqlite::memory:` WAL is a documented no-op (the
    // PRAGMA returns "memory"); neither statement errors there.
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(pool)
        .await?;

    // Core migrations first, into the default `_sqlx_migrations` table. A plugin
    // may FK only to already-migrated core tables; core never depends on a plugin.
    sqlx::migrate!("./migrations").run(pool).await?;

    // Then each plugin, each into its own keyspace so their versions can't collide.
    for plugin in &plugins {
        let Some(mut migrator) = plugin.migrator() else {
            continue;
        };
        // The host -- not the plugin -- sets the tracking-table name, derived
        // from the plugin name (§5); `validate_registry` already proved the name
        // is a safe `^[a-z][a-z0-9_]*$` identifier.
        migrator.dangerous_set_table_name(format!("_sqlx_migrations_{}", plugin.name()));
        // §5.4: on partial failure, abort startup naming the failing plugin.
        // Earlier plugins stay applied (each owns its keyspace); a re-run resumes.
        migrator
            .run(pool)
            .await
            .with_context(|| format!("plugin '{}' migrations failed", plugin.name()))?;
    }
    Ok(())
}

/// Refuses a plugin whose declared `host_api` range is incompatible with this
/// host's [`PLUGIN_API_VERSION`], or whose `name` is not a safe identifier. The
/// name derives the route prefix (`/api/v1/<name>`), the OpenAPI component prefix
/// (`<name>_*`), and the migration tracking-table name (`_sqlx_migrations_<name>`),
/// so a malformed name would yield a broken router or an invalid SQL identifier.
/// Called once at startup before any migration runs, so an incompatible build
/// fails loudly with a human-readable error (docs/plugin-framework.md §3).
fn validate_registry(plugins: &[Box<dyn Plugin>]) -> anyhow::Result<()> {
    let host = semver::Version::parse(PLUGIN_API_VERSION).with_context(|| {
        format!("PLUGIN_API_VERSION '{PLUGIN_API_VERSION}' is not valid semver")
    })?;
    for plugin in plugins {
        check_plugin(plugin.name(), plugin.host_api(), &host)?;
    }
    Ok(())
}

/// The per-plugin half of [`validate_registry`], split out as a pure function so
/// the name-format and version-compatibility rules can be unit-tested without a
/// live registry. The name is checked first, since the range error message is
/// less useful when the name itself is malformed.
fn check_plugin(name: &str, host_api: &str, host: &semver::Version) -> anyhow::Result<()> {
    anyhow::ensure!(
        is_valid_plugin_name(name),
        "plugin name '{name}' must match ^[a-z][a-z0-9_]*$: it derives the route, \
         schema, and migration-table names"
    );
    let req = semver::VersionReq::parse(host_api).with_context(|| {
        format!("plugin '{name}' declared an invalid host_api range '{host_api}'")
    })?;
    anyhow::ensure!(
        req.matches(host),
        "plugin '{name}' requires host_api '{host_api}', incompatible with this host's \
         PLUGIN_API_VERSION {host}"
    );
    Ok(())
}

/// Whether `name` is a valid plugin namespace key: `^[a-z][a-z0-9_]*$`. Public
/// (and matched by the `just new-plugin` scaffolder's own regex) because the §6
/// guard test asserts the same shape over the real registry.
pub fn is_valid_plugin_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Redacts any `user:password@` userinfo from a connection string so it is safe
/// to log. The default SQLite URL has no userinfo and is returned unchanged; a
/// Postgres `DATABASE_URL` carries the password inline, and it must never reach
/// the logs.
pub fn redact_url(database_url: &str) -> String {
    let Some((scheme, rest)) = database_url.split_once("://") else {
        return database_url.to_owned();
    };
    // Userinfo, when present, sits between "://" and the last '@' before the
    // authority ends -- at the first '/' (path) or '?' (query). URLs without an
    // authority '@' (every `sqlite://` URL) are returned unchanged.
    let authority_end = rest.find(['/', '?']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(authority_end);
    match authority.rsplit_once('@') {
        Some((_userinfo, host)) => format!("{scheme}://***@{host}{tail}"),
        None => database_url.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_plugin_names() {
        for name in ["todo", "blog", "guest_book", "a", "a1", "x_9_y"] {
            assert!(is_valid_plugin_name(name), "{name:?} should be valid");
        }
    }

    #[test]
    fn rejects_invalid_plugin_names() {
        // Leading non-letter, uppercase, hyphen, space, punctuation, non-ascii, empty.
        for name in [
            "", "Todo", "1todo", "_todo", "to-do", "to do", "todo!", "tödo",
        ] {
            assert!(!is_valid_plugin_name(name), "{name:?} should be rejected");
        }
    }

    #[test]
    fn check_plugin_accepts_compatible_ranges() {
        let host = semver::Version::parse("1.0.0").unwrap();
        assert!(check_plugin("todo", "^1", &host).is_ok());
        assert!(check_plugin("blog", "^1.0", &host).is_ok());
        assert!(check_plugin("x", ">=1, <2", &host).is_ok());
    }

    #[test]
    fn check_plugin_rejects_incompatible_range() {
        let host = semver::Version::parse("1.0.0").unwrap();
        let err = check_plugin("todo", "^2", &host).unwrap_err().to_string();
        assert!(err.contains("incompatible"), "unexpected error: {err}");
    }

    #[test]
    fn check_plugin_rejects_unparseable_range() {
        let host = semver::Version::parse("1.0.0").unwrap();
        assert!(check_plugin("todo", "not-a-range", &host).is_err());
    }

    #[test]
    fn check_plugin_rejects_bad_name_before_range() {
        // A malformed name fails even when paired with an otherwise valid range.
        let host = semver::Version::parse("1.0.0").unwrap();
        let err = check_plugin("Bad-Name", "^1", &host)
            .unwrap_err()
            .to_string();
        assert!(err.contains("must match"), "unexpected error: {err}");
    }

    #[test]
    fn registered_plugins_satisfy_the_contract() {
        validate_registry(&crate::plugins::all()).expect("the real registry must validate");
    }

    #[test]
    fn leaves_sqlite_urls_untouched() {
        assert_eq!(
            redact_url("sqlite://data/app.db?mode=rwc"),
            "sqlite://data/app.db?mode=rwc"
        );
        assert_eq!(redact_url("sqlite::memory:"), "sqlite::memory:");
    }

    #[test]
    fn redacts_userinfo_from_network_urls() {
        assert_eq!(
            redact_url("postgres://user:pass@localhost:5432/db"),
            "postgres://***@localhost:5432/db"
        );
        // Password containing a literal '@' splits on the last authority '@'.
        assert_eq!(
            redact_url("postgres://user:p@ss@host/db?sslmode=require"),
            "postgres://***@host/db?sslmode=require"
        );
        // No path component still redacts the userinfo.
        assert_eq!(
            redact_url("mysql://root:secret@host:3306"),
            "mysql://***@host:3306"
        );
        // A query-string '@' on a path-less URL is not mistaken for userinfo.
        assert_eq!(
            redact_url("postgres://user:pass@host?opt=a@b"),
            "postgres://***@host?opt=a@b"
        );
    }
}
