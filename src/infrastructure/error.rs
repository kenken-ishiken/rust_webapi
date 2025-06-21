use actix_web::{error::ResponseError, HttpResponse};
use thiserror::Error;
// use std::fmt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    // #[error("Bad request: {0}")]
    // BadRequest(String),

    // #[error("Unauthorized: {0}")]
    // Unauthorized(String),

    // #[error("Forbidden: {0}")]
    // Forbidden(String),

    // #[error("Conflict: {0}")]
    // Conflict(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    // #[error("Service unavailable: {0}")]
    // ServiceUnavailable(String),

    // #[error("Validation error: {0}")]
    // ValidationError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, message) = match self {
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "データベースエラーが発生しました".to_string(),
                )
            }
            AppError::NotFound(msg) => (
                actix_web::http::StatusCode::NOT_FOUND,
                "not_found",
                msg.clone(),
            ),
            // AppError::BadRequest(msg) => (
            //     actix_web::http::StatusCode::BAD_REQUEST,
            //     "bad_request",
            //     msg.clone(),
            // ),
            // AppError::Unauthorized(msg) => (
            //     actix_web::http::StatusCode::UNAUTHORIZED,
            //     "unauthorized",
            //     msg.clone(),
            // ),
            // AppError::Forbidden(msg) => (
            //     actix_web::http::StatusCode::FORBIDDEN,
            //     "forbidden",
            //     msg.clone(),
            // ),
            // AppError::Conflict(msg) => (
            //     actix_web::http::StatusCode::CONFLICT,
            //     "conflict",
            //     msg.clone(),
            // ),
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "サーバーエラーが発生しました".to_string(),
                )
            }
            // AppError::ServiceUnavailable(msg) => (
            //     actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            //     "service_unavailable",
            //     msg.clone(),
            // ),
            // AppError::ValidationError(msg) => (
            //     actix_web::http::StatusCode::BAD_REQUEST,
            //     "validation_error",
            //     msg.clone(),
            // ),
            AppError::AuthenticationError(msg) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "authentication_error",
                msg.clone(),
            ),
            AppError::ExternalServiceError(msg) => {
                tracing::error!("External service error: {}", msg);
                (
                    actix_web::http::StatusCode::BAD_GATEWAY,
                    "external_service_error",
                    msg.clone(),
                )
            }
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        }))
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Conversion helpers
impl AppError {
    // pub fn not_found(entity: &str, id: impl fmt::Display) -> Self {
    //     AppError::NotFound(format!("{} with id {} not found", entity, id))
    // }

    // pub fn bad_request(msg: impl Into<String>) -> Self {
    //     AppError::BadRequest(msg.into())
    // }

    // pub fn unauthorized(msg: impl Into<String>) -> Self {
    //     AppError::Unauthorized(msg.into())
    // }

    // pub fn internal_error(msg: impl Into<String>) -> Self {
    //     AppError::InternalServerError(msg.into())
    // }

    // pub fn validation_error(msg: impl Into<String>) -> Self {
    //     AppError::ValidationError(msg.into())
    // }
}

// Convert from other error types
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::AuthenticationError(format!("JWT error: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ExternalServiceError(format!("HTTP client error: {}", err))
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::InternalServerError(format!("Environment variable error: {}", err))
    }
}
