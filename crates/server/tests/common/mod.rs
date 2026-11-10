#![allow(dead_code)] // each test binary uses a different subset of these helpers

//! Shared helpers for API tests: build the real router on a test DB and send requests.

use std::{path::Path, time::Duration};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use server::{app, AppState};
use sqlx::PgPool;
use tower::ServiceExt;

/// The real router, backed by the given (per-test) pool. No SPA dir. The workspace is a
/// fresh temp dir that is kept for the life of the test, so nothing touches the real filesystem.
pub fn test_app(pool: PgPool) -> Router {
    let dir = tempfile::TempDir::new().unwrap().keep();
    test_app_in(pool, &dir)
}

/// The real router with a given workspace dir (use a per-test temp dir).
pub fn test_app_in(pool: PgPool, workspace_dir: &Path) -> Router {
    app(
        AppState {
            pool,
            workspace_dir: workspace_dir.to_path_buf(),
            clone_timeout: Duration::from_secs(600),
        },
        Path::new("/nonexistent-web-dist"),
    )
}

/// Send one request. Returns the status and the body as JSON
/// (or a JSON string holding the raw text if the body is not JSON).
pub async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<String>,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.unwrap_or_default()))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };
    (status, json)
}
