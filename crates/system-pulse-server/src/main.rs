use anyhow::{Context, Result};
use system_pulse_db::{Database, DatabaseConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use system_pulse_server::config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env().context("failed to load config from environment")?;

    // DatabaseConfig::from_env() reads DATABASE_PATH, defaulting to
    // /data/system_pulse.db — the path mounted as a Docker volume.
    let db_config = DatabaseConfig::from_env();
    tracing::info!(path = %db_config.path.display(), "starting system-pulse-server");

    let db = Database::connect(db_config).await.context("failed to connect to database")?;

    let app = system_pulse_server::build_router(db, config.clone());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(addr = %addr, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
