//! D. API tests for workspace projects: real router, real Postgres, temp workspace dir.
//! No network: the git URL used is `example.invalid`, and only the response is checked.
//! Rows are arranged with raw SQL so each test isolates the behaviour it checks.

mod common;

use axum::{http::StatusCode, Router};
use serde_json::json;
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

use common::{send, test_app_in};

const BASE: &str = "/api/v1/projects";

/// Arrange a row with raw SQL. `source` and `status` are literal enum values.
async fn raw_insert(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    source: &str,
    status: &str,
    message: Option<&str>,
) {
    sqlx::query(&format!(
        "INSERT INTO projects (id, name, repo_url, source, status, status_message) \
         VALUES ($1, $2, $3, '{source}', '{status}', $4)"
    ))
    .bind(id)
    .bind(name)
    .bind((source == "git").then_some("https://example.invalid/x.git"))
    .bind(message)
    .execute(pool)
    .await
    .unwrap();
}

fn app_with(pool: &PgPool, ws: &TempDir) -> Router {
    test_app_in(pool.clone(), ws.path())
}

// ---- POST /projects -----------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn post_without_repo_url_returns_201_ready_scratch_with_git_folder(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let app = app_with(&pool, &ws);
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(json!({"name": "Scratch"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "ready");
    assert_eq!(body["source"], "scratch");
    let id = body["id"].as_str().unwrap();
    assert!(ws.path().join(id).join(".git").is_dir());
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_with_repo_url_returns_201_cloning_git(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let app = app_with(&pool, &ws);
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(json!({"name": "Imported", "repo_url": "https://example.invalid/x.git"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "cloning");
    assert_eq!(body["source"], "git");
}

// ---- POST /projects/{id}/retry ------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn retry_failed_git_project_returns_202_cloning(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(
        &pool,
        id,
        "Imported",
        "git",
        "failed",
        Some("clone timed out"),
    )
    .await;
    let app = app_with(&pool, &ws);
    let (status, body) = send(&app, "POST", &format!("{BASE}/{id}/retry"), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "cloning");
}

#[sqlx::test(migrations = "../../migrations")]
async fn retry_ready_scratch_project_returns_409(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(&pool, id, "Scratch", "scratch", "ready", None).await;
    let app = app_with(&pool, &ws);
    let (status, body) = send(&app, "POST", &format!("{BASE}/{id}/retry"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}

#[sqlx::test(migrations = "../../migrations")]
async fn retry_ready_git_project_returns_409(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(&pool, id, "Imported", "git", "ready", None).await;
    let app = app_with(&pool, &ws);
    let (status, body) = send(&app, "POST", &format!("{BASE}/{id}/retry"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}

// ---- DELETE /projects/{id} ----------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_project_folder(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(&pool, id, "Scratch", "scratch", "ready", None).await;
    let folder = ws.path().join(id.to_string());
    std::fs::create_dir_all(folder.join(".git")).unwrap();
    let app = app_with(&pool, &ws);
    let (status, _) = send(&app, "DELETE", &format!("{BASE}/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(!folder.exists());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_when_folder_already_gone_returns_204(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(&pool, id, "Scratch", "scratch", "ready", None).await;
    let app = app_with(&pool, &ws);
    let (status, _) = send(&app, "DELETE", &format!("{BASE}/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(&app, "GET", &format!("{BASE}/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---- startup recovery ----------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn startup_marks_cloning_project_failed_interrupted_and_removes_folder(pool: PgPool) {
    let ws = TempDir::new().unwrap();
    let id = Uuid::now_v7();
    raw_insert(&pool, id, "Imported", "git", "cloning", None).await;
    let folder = ws.path().join(id.to_string());
    std::fs::create_dir_all(folder.join(".git")).unwrap();

    server::recover_interrupted_clones(&pool, ws.path())
        .await
        .unwrap();

    let row = db::get_project(&pool, id).await.unwrap().unwrap();
    assert_eq!(row.status, api_types::ProjectStatus::Failed);
    assert_eq!(row.status_message.as_deref(), Some("interrupted"));
    assert!(!folder.exists());
}
