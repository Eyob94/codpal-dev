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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectSource {
    Scratch,
    Git,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectStatus {
    Cloning,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub repo_url: Option<String>,
    pub description: Option<String>,
    pub source: ProjectSource,
    pub status: ProjectStatus,
    pub status_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validated partial update, produced by core. `None` = leave the field alone;
/// `Some(None)` = clear it; `Some(Some(v))` = set it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectPatch {
    pub name: Option<String>,
    pub repo_url: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

// ---- intents (user actions sent to Rust) ----------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateItem {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateProject {
    pub name: String,
    pub repo_url: Option<String>,
    pub description: Option<String>,
}

/// Every field optional. An empty string clears `repo_url` / `description`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub description: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub repo_url: Option<String>,
    pub description: Option<String>,
    pub source: ProjectSource,
    pub status: ProjectStatus,
    /// Short reason when `status` is `failed`.
    pub status_message: Option<String>,
    /// True only for a `git` project whose clone failed.
    pub can_retry: bool,
    /// RFC 3339, UTC.
    pub created_at: String,
    /// RFC 3339, UTC.
    pub updated_at: String,
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
    Conflict,
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
