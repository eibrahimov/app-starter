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
    for plugin in crate::plugins::all() {
        let Some(mut migrator) = plugin.migrator() else {
            continue;
        };
        // The host -- not the plugin -- sets the tracking-table name, derived
        // from the validated plugin name (§5).
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
