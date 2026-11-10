//! C. API tests: real router + real Postgres (fresh DB per test). Status + body shape.

mod common;

use axum::{http::StatusCode, Router};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use common::{send, test_app};

const BASE: &str = "/api/v1/projects";

fn item_uri(id: &str) -> String {
    format!("{BASE}/{id}")
}

fn field_names(body: &Value) -> Vec<String> {
    body["fields"]
        .as_array()
        .expect("fields array")
        .iter()
        .map(|f| f["field"].as_str().expect("field name").to_owned())
        .collect()
}

/// Arrange helper: create a project by name and return its body.
async fn create(app: &Router, name: &str) -> Value {
    let (status, body) = send(app, "POST", BASE, Some(json!({ "name": name }).to_string())).await;
    assert_eq!(status, StatusCode::CREATED);
    body
}

// ---- POST /projects -----------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn post_valid_returns_201_with_project_view(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(
            json!({"name": "Alpha", "repo_url": "https://github.com/a/b", "description": "desc"})
                .to_string(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Alpha");
    assert_eq!(body["repo_url"], "https://github.com/a/b");
    assert_eq!(body["description"], "desc");
    Uuid::parse_str(body["id"].as_str().unwrap()).expect("id is a UUID string");
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_then_get_returns_same_project(pool: PgPool) {
    let app = test_app(pool);
    let created = create(&app, "Alpha").await;
    let id = created["id"].as_str().unwrap();
    let (status, fetched) = send(&app, "GET", &item_uri(id), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched, created);
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_invalid_returns_400_with_each_bad_field(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(json!({"name": "", "repo_url": "bad"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
    assert_eq!(field_names(&body), vec!["name", "repo_url"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_duplicate_name_returns_409_conflict(pool: PgPool) {
    let app = test_app(pool);
    create(&app, "Alpha").await;
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(json!({"name": "Alpha"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_malformed_json_returns_api_error_body(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(&app, "POST", BASE, Some("{not json".into())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
    assert!(body["message"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn post_missing_name_returns_api_error_body(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(
        &app,
        "POST",
        BASE,
        Some(json!({"repo_url": null}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
    assert!(body["message"].is_string());
}

// ---- GET /projects and GET /projects/{id} -------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn get_projects_empty_returns_200_empty_array(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(&app, "GET", BASE, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_projects_lists_newest_first(pool: PgPool) {
    let app = test_app(pool);
    create(&app, "first").await;
    create(&app, "second").await;
    let (status, body) = send(&app, "GET", BASE, None).await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["second", "first"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_unknown_id_returns_404_not_found(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(&app, "GET", &item_uri(&Uuid::now_v7().to_string()), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_non_uuid_id_returns_404_not_found(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(&app, "GET", &item_uri("not-a-uuid"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

// ---- PATCH /projects/{id} -----------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn patch_valid_returns_200_updated_view(pool: PgPool) {
    let app = test_app(pool);
    let created = create(&app, "Alpha").await;
    let id = created["id"].as_str().unwrap();
    let (status, body) = send(
        &app,
        "PATCH",
        &item_uri(id),
        Some(json!({"name": "Renamed"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], created["id"]);
    assert_eq!(body["name"], "Renamed");
    assert_eq!(body["repo_url"], created["repo_url"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn patch_invalid_returns_400_validation(pool: PgPool) {
    let app = test_app(pool);
    let created = create(&app, "Alpha").await;
    let id = created["id"].as_str().unwrap();
    let (status, body) = send(
        &app,
        "PATCH",
        &item_uri(id),
        Some(json!({"name": ""}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
    assert_eq!(field_names(&body), vec!["name"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn patch_unknown_id_returns_404(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(
        &app,
        "PATCH",
        &item_uri(&Uuid::now_v7().to_string()),
        Some(json!({"name": "x"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn patch_rename_to_existing_name_returns_409(pool: PgPool) {
    let app = test_app(pool);
    create(&app, "A").await;
    let b = create(&app, "B").await;
    let id = b["id"].as_str().unwrap();
    let (status, body) = send(
        &app,
        "PATCH",
        &item_uri(id),
        Some(json!({"name": "A"}).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}

// ---- DELETE /projects/{id} ----------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn delete_returns_204_then_get_returns_404(pool: PgPool) {
    let app = test_app(pool);
    let created = create(&app, "Alpha").await;
    let uri = item_uri(created["id"].as_str().unwrap());
    let (status, _) = send(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = send(&app, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_unknown_id_returns_404(pool: PgPool) {
    let app = test_app(pool);
    let (status, body) = send(&app, "DELETE", &item_uri(&Uuid::now_v7().to_string()), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], "not_found");
}

// ---- no route returns 500 for user input --------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn bad_input_never_returns_500(pool: PgPool) {
    let app = test_app(pool);
    let cases: [(&str, &str, Option<String>, StatusCode); 5] = [
        // NUL byte: Postgres cannot store it, so it must be a validation error.
        (
            "POST",
            BASE,
            Some(json!({"name": "a\u{0}b"}).to_string()),
            StatusCode::BAD_REQUEST,
        ),
        ("GET", &item_uri("not-a-uuid"), None, StatusCode::NOT_FOUND),
        (
            "PATCH",
            &item_uri("not-a-uuid"),
            Some(json!({"name": "x"}).to_string()),
            StatusCode::NOT_FOUND,
        ),
        (
            "DELETE",
            &item_uri("not-a-uuid"),
            None,
            StatusCode::NOT_FOUND,
        ),
        (
            "POST",
            BASE,
            Some("{not json".into()),
            StatusCode::BAD_REQUEST,
        ),
    ];
    for (method, uri, body, expected) in cases {
        let (status, _) = send(&app, method, uri, body).await;
        assert_eq!(status, expected, "{method} {uri}");
    }
}
