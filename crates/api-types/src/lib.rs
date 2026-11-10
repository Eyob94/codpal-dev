//! Wire types shared by server, core and wasm. Every type here is exported to
//! TypeScript via ts-rs; the frontend must never hand-write API types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

// ---- domain / persistence shape -------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Item {
    pub id: Uuid,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

// ---- intents (user actions sent to Rust) ----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateItem {
    pub title: String,
}

// ---- view models (ready-to-render, returned to React) ---------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ItemView {
    pub id: String,
    pub label: String,
    pub badge: Badge,
    pub can_complete: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Badge {
    Open,
    Done,
}

// ---- errors ---------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorCode {
    Validation,
    NotFound,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub fields: Vec<FieldError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Health {
    pub status: String,
    pub db: bool,
}
