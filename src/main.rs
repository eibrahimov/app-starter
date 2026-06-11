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
    tracing::info!(port = args.port, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}
