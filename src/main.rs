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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();
    let pool = app_starter::db::init(&args.database_url).await?;
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
