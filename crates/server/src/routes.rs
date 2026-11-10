use api_types::{CreateItem, CreateProject, Health, ItemView, ProjectView, UpdateProject};
use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{error::AppError, AppState};

pub fn api() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}/complete", post(complete_item))
        .route("/projects", get(list_projects).post(create_project))
        .route(
            "/projects/{id}",
            get(get_project)
                .patch(update_project)
                .delete(delete_project),
        )
}

// Handlers stay thin: parse -> call core/db -> map to view model.

async fn health(State(s): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok".into(),
        db: db::ping(&s.pool).await,
    })
}

async fn list_items(State(s): State<AppState>) -> Result<Json<Vec<ItemView>>, AppError> {
    let items = db::list_items(&s.pool).await.map_err(AppError::internal)?;
    Ok(Json(items.iter().map(core_logic::item_view).collect()))
}

async fn create_item(
    State(s): State<AppState>,
    Json(input): Json<CreateItem>,
) -> Result<(StatusCode, Json<ItemView>), AppError> {
    let input = core_logic::validate_create_item(&input).map_err(AppError::Validation)?;
    let item = db::insert_item(&s.pool, &input.title)
        .await
        .map_err(AppError::internal)?;
    Ok((StatusCode::CREATED, Json(core_logic::item_view(&item))))
}

async fn complete_item(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ItemView>, AppError> {
    let item = db::complete_item(&s.pool, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(core_logic::item_view(&item)))
}

// ---- projects (interface only; bodies arrive in the Code issue #3) ----------
// `id` is a String so a malformed id maps to our ApiError 404, not axum's plain-text 400.
// The body is a Result so a malformed JSON body maps to our ApiError, not axum's plain text.

async fn list_projects(State(_s): State<AppState>) -> Result<Json<Vec<ProjectView>>, AppError> {
    todo!("Code issue #3")
}

async fn create_project(
    State(_s): State<AppState>,
    _body: Result<Json<CreateProject>, JsonRejection>,
) -> Result<(StatusCode, Json<ProjectView>), AppError> {
    todo!("Code issue #3")
}

async fn get_project(
    State(_s): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<ProjectView>, AppError> {
    todo!("Code issue #3")
}

async fn update_project(
    State(_s): State<AppState>,
    Path(_id): Path<String>,
    _body: Result<Json<UpdateProject>, JsonRejection>,
) -> Result<Json<ProjectView>, AppError> {
    todo!("Code issue #3")
}

async fn delete_project(
    State(_s): State<AppState>,
    Path(_id): Path<String>,
) -> Result<StatusCode, AppError> {
    todo!("Code issue #3")
}
