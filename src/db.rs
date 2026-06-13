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
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
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
