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
