use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub web_dist: PathBuf,
    pub run_migrations: bool,
    /// Root folder for all project folders (`CODPAL_WORKSPACE_DIR`, default `./workspace`).
    pub workspace_dir: PathBuf,
    /// Git clone timeout (`CODPAL_CLONE_TIMEOUT_SECS`, default 600).
    pub clone_timeout: Duration,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            bind_addr: env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".into())
                .parse()?,
            web_dist: env::var("WEB_DIST")
                .unwrap_or_else(|_| "web/dist".into())
                .into(),
            run_migrations: env::var("RUN_MIGRATIONS")
                .map(|v| v != "false")
                .unwrap_or(true),
            workspace_dir: env::var("CODPAL_WORKSPACE_DIR")
                .unwrap_or_else(|_| "./workspace".into())
                .into(),
            clone_timeout: Duration::from_secs(
                env::var("CODPAL_CLONE_TIMEOUT_SECS")
                    .ok()
                    .map(|v| v.parse())
                    .transpose()?
                    .unwrap_or(600),
            ),
        })
    }
}
