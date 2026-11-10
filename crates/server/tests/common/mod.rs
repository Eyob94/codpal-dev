//! Shared helpers for API tests: build the real router on a test DB and send requests.

use std::path::Path;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use server::{app, AppState};
use sqlx::PgPool;
use tower::ServiceExt;

/// The real router, backed by the given (per-test) pool. No SPA dir.
pub fn test_app(pool: PgPool) -> Router {
    app(AppState { pool }, Path::new("/nonexistent-web-dist"))
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
