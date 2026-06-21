use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "app-starter", version, about = "App Starter server")]
struct Args {
    /// Port to bind the HTTP server to.
    #[arg(long, env = "PORT", default_value_t = 8080)]
    port: u16,

    /// SQLite connection string. mode=rwc creates the file on first run.
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "sqlite://data/app.db?mode=rwc"
    )]
    database_url: String,

    /// Seed the database with example items and posts on startup, then keep
    /// serving. Off by default; also enabled with `SEED=1`. Idempotent for a
    /// single seeding process — it skips any resource that already has rows —
    /// so it is safe to leave on for a normal single-instance deploy.
    /// Removable convenience — see `src/seed.rs`.
    #[arg(long)]
    seed: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();
    let pool = app_starter::db::init(&args.database_url).await?;

    // Optional, deletable seed seam: populate a fresh database with example
    // rows so the UI demos immediately. Off by default; see `src/seed.rs`.
    if args.seed || std::env::var("SEED").is_ok_and(|v| is_truthy(&v)) {
        app_starter::seed::run(&pool).await?;
    }

    let app = app_starter::api::router(app_starter::AppState { pool });

    let listener = TcpListener::bind(("0.0.0.0", args.port)).await?;
    tracing::info!(
        port = args.port,
        database = %app_starter::db::redact_url(&args.database_url),
        "listening"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// True when `value` is a recognised truthy spelling (`1`, `true`, `yes`,
/// `on`; case-insensitive, surrounding whitespace ignored). Used for the
/// `SEED` environment toggle so `SEED=1` enables seeding, matching `--seed`.
/// clap's bool flag parser only accepts `true`/`false`, so the env side is
/// handled here rather than via `#[arg(env = "SEED")]`.
fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Resolves when the process receives Ctrl-C or (on Unix) SIGTERM, so axum
/// drains in-flight requests before stopping. This matters most for SQLite:
/// a hard kill mid-write can leave the database locked.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, draining connections");
}

#[cfg(test)]
mod tests {
    use super::is_truthy;

    #[test]
    fn truthy_accepts_common_spellings() {
        for value in ["1", "true", "TRUE", "Yes", "on", "  true  "] {
            assert!(is_truthy(value), "{value:?} should enable seeding");
        }
    }

    #[test]
    fn truthy_rejects_everything_else() {
        for value in ["0", "false", "no", "off", "", "2", "maybe"] {
            assert!(!is_truthy(value), "{value:?} should not enable seeding");
        }
    }
}
