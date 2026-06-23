use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use system_pulse_db::DbError;

pub enum ApiError {
    NotFound(String),
    Unauthorized(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, kind, message) = match self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", m),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, "CONFLICT", m),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", m),
            ApiError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", "internal server error".to_string())
            }
        };
        (status, Json(ErrBody { error: kind, message })).into_response()
    }
}

impl From<DbError> for ApiError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::NotFound(m) => ApiError::NotFound(m),
            DbError::Conflict(m) => ApiError::Conflict(m),
            other => ApiError::Internal(other.to_string()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
