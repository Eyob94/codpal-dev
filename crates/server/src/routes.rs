use api_types::{CreateItem, Health, ItemView};
use axum::{
    extract::{Path, State},
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
