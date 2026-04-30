use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Unauthorized,
    Internal(String),
    BadRequest(String),
    Core(tokimo_core::CoreError),
}

impl From<tokimo_core::CoreError> for AppError {
    fn from(e: tokimo_core::CoreError) -> Self {
        AppError::Core(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Core(e) => {
                tracing::error!("Core error: {:?}", e);
                match e {
                    tokimo_core::CoreError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
                    tokimo_core::CoreError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limited".to_string()),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
                }
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
