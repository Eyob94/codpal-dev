use server::{app, config::Config, AppState};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let cfg = Config::from_env()?;
    let pool = db::connect(&cfg.database_url).await?;
    if cfg.run_migrations {
        db::migrate(&pool).await?;
    }

    let listener = tokio::net::TcpListener::bind(cfg.bind_addr).await?;
    tracing::info!("listening on {}", cfg.bind_addr);
    axum::serve(listener, app(AppState { pool }, &cfg.web_dist))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
