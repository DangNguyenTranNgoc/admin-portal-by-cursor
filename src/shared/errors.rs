use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("user not found")]
    UserNotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid status `{0}`")]
    InvalidStatus(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("database error: {0}")]
    Database(String),
    #[error("kafka error: {0}")]
    Kafka(String),
    #[error("clickhouse error: {0}")]
    ClickHouse(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{message}")]
    BadRequest { message: String },
    #[error("{message}")]
    Unauthorized { message: String },
    #[error("{message}")]
    Forbidden { message: String },
    #[error("{message}")]
    NotFound { message: String },
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest { message } => (StatusCode::BAD_REQUEST, message),
            ApiError::Unauthorized { message } => (StatusCode::UNAUTHORIZED, message),
            ApiError::Forbidden { message } => (StatusCode::FORBIDDEN, message),
            ApiError::NotFound { message } => (StatusCode::NOT_FOUND, message),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };

        let body = ErrorBody { message };
        (status, Json(body)).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl From<DomainError> for ApiError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::UserNotFound => ApiError::NotFound {
                message: "user not found".to_string(),
            },
            DomainError::InvalidCredentials => ApiError::Unauthorized {
                message: "invalid credentials".to_string(),
            },
            DomainError::PermissionDenied => ApiError::Forbidden {
                message: "forbidden".to_string(),
            },
            DomainError::InvalidStatus(msg) => ApiError::BadRequest { message: msg },
            DomainError::Unexpected(_msg) => ApiError::Internal,
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Internal error: {err:?}");
        ApiError::Internal
    }
}
