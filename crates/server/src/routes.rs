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

// ---- projects -----------------------------------------------------------------
// `id` is a String so a malformed id maps to our ApiError 404, not axum's plain-text 400.
// The body is a Result so a malformed JSON body maps to our ApiError, not axum's plain text.

fn parse_id(raw: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(raw).map_err(|_| AppError::NotFound)
}

async fn list_projects(State(s): State<AppState>) -> Result<Json<Vec<ProjectView>>, AppError> {
    let projects = db::list_projects(&s.pool).await?;
    Ok(Json(
        projects.iter().map(core_logic::project_view).collect(),
    ))
}

async fn create_project(
    State(s): State<AppState>,
    body: Result<Json<CreateProject>, JsonRejection>,
) -> Result<(StatusCode, Json<ProjectView>), AppError> {
    let Json(input) = body?;
    let input = core_logic::validate_create_project(&input).map_err(AppError::Validation)?;
    let project = db::insert_project(&s.pool, &input).await?;
    Ok((
        StatusCode::CREATED,
        Json(core_logic::project_view(&project)),
    ))
}

async fn get_project(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectView>, AppError> {
    let project = db::get_project(&s.pool, parse_id(&id)?)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(core_logic::project_view(&project)))
}

async fn update_project(
    State(s): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<UpdateProject>, JsonRejection>,
) -> Result<Json<ProjectView>, AppError> {
    let id = parse_id(&id)?;
    let Json(input) = body?;
    let patch = core_logic::validate_update_project(&input).map_err(AppError::Validation)?;
    let project = db::update_project(&s.pool, id, &patch)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(core_logic::project_view(&project)))
}

async fn delete_project(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if db::delete_project(&s.pool, parse_id(&id)?).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
