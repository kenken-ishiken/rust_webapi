use actix_web::{error::ResponseError, HttpResponse};
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
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
            AppError::BadRequest(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "bad_request",
                msg.clone(),
            ),
            AppError::Unauthorized(msg) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "unauthorized",
                msg.clone(),
            ),
            AppError::Forbidden(msg) => (
                actix_web::http::StatusCode::FORBIDDEN,
                "forbidden",
                msg.clone(),
            ),
            AppError::Conflict(msg) => (
                actix_web::http::StatusCode::CONFLICT,
                "conflict",
                msg.clone(),
            ),
            AppError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "サーバーエラーが発生しました".to_string(),
                )
            }
            AppError::ServiceUnavailable(msg) => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "service_unavailable",
                msg.clone(),
            ),
            AppError::ValidationError(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "validation_error",
                msg.clone(),
            ),
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
            AppError::ConfigurationError(msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "configuration_error",
                    "設定エラーが発生しました".to_string(),
                )
            }
            AppError::SerializationError(msg) => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "serialization_error",
                format!("データの変換に失敗しました: {}", msg),
            ),
            AppError::NetworkError(msg) => {
                tracing::error!("Network error: {}", msg);
                (
                    actix_web::http::StatusCode::BAD_GATEWAY,
                    "network_error",
                    "ネットワークエラーが発生しました".to_string(),
                )
            }
            AppError::TimeoutError(msg) => (
                actix_web::http::StatusCode::REQUEST_TIMEOUT,
                "timeout_error",
                format!("タイムアウトが発生しました: {}", msg),
            ),
            AppError::Generic(e) => {
                tracing::error!("Generic error: {:?}", e);
                (
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "generic_error",
                    "予期しないエラーが発生しました".to_string(),
                )
            }
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "type": error_type,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        }))
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Conversion helpers
impl AppError {
    /// エンティティが見つからない場合のエラーを生成
    pub fn not_found(entity: &str, id: impl fmt::Display) -> Self {
        AppError::NotFound(format!("{} with id {} not found", entity, id))
    }

    /// バリデーションエラーを生成
    pub fn validation_error(msg: impl Into<String>) -> Self {
        AppError::ValidationError(msg.into())
    }

    /// 不正なリクエストエラーを生成
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    /// 認証エラーを生成
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized(msg.into())
    }

    /// 権限エラーを生成
    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }

    /// 競合エラーを生成
    pub fn conflict(msg: impl Into<String>) -> Self {
        AppError::Conflict(msg.into())
    }

    /// 内部サーバーエラーを生成
    pub fn internal_error(msg: impl Into<String>) -> Self {
        AppError::InternalServerError(msg.into())
    }

    /// 設定エラーを生成
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        AppError::ConfigurationError(msg.into())
    }

    /// シリアライゼーションエラーを生成
    pub fn serialization_error(msg: impl Into<String>) -> Self {
        AppError::SerializationError(msg.into())
    }

    /// ネットワークエラーを生成
    pub fn network_error(msg: impl Into<String>) -> Self {
        AppError::NetworkError(msg.into())
    }

    /// タイムアウトエラーを生成
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        AppError::TimeoutError(msg.into())
    }

    /// 汎用エラーを生成（anyhowから）
    pub fn from_anyhow(err: anyhow::Error) -> Self {
        AppError::Generic(err)
    }
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
        AppError::ConfigurationError(format!("Environment variable error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::NetworkError(format!("IO error: {}", err))
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AppError::TimeoutError(format!("Timeout: {}", err))
    }
}
