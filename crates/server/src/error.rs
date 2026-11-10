use api_types::{ApiError, ErrorCode, FieldError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation failed")]
    Validation(Vec<FieldError>),
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn internal(e: impl std::fmt::Display) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::Validation(fields) => (
                StatusCode::BAD_REQUEST,
                ApiError {
                    code: ErrorCode::Validation,
                    message: "Validation failed".into(),
                    fields,
                },
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                ApiError {
                    code: ErrorCode::NotFound,
                    message: "Not found".into(),
                    fields: vec![],
                },
            ),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                ApiError {
                    code: ErrorCode::Conflict,
                    message: "Conflict".into(),
                    fields: vec![],
                },
            ),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError {
                        code: ErrorCode::Internal,
                        message: "Internal error".into(),
                        fields: vec![],
                    },
                )
            }
        };
        (status, Json(body)).into_response()
    }
}
