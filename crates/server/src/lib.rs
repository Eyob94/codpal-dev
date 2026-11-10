pub mod config;
pub mod error;
pub mod routes;

use std::{path::Path, time::Duration};

use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: db::Pool,
    /// Root folder for project folders.
    pub workspace_dir: std::path::PathBuf,
    /// Timeout for a git clone.
    pub clone_timeout: Duration,
}

/// Startup recovery: every `cloning` project becomes `failed` ("interrupted"), and its
/// half-cloned folder under `workspace_dir` is removed.
pub async fn recover_interrupted_clones(
    _pool: &db::Pool,
    _workspace_dir: &Path,
) -> anyhow::Result<()> {
    todo!()
}

pub fn app(state: AppState, web_dist: &Path) -> Router {
    let router = Router::new()
        .nest("/api/v1", routes::api())
        .with_state(state);

    // Serve the built SPA (with index.html fallback) when it exists.
    let router = if web_dist.join("index.html").exists() {
        let spa = ServeDir::new(web_dist).fallback(ServeFile::new(web_dist.join("index.html")));
        router.fallback_service(spa)
    } else {
        router
    };

    router
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
}
