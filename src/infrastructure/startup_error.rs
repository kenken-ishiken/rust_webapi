use thiserror::Error;

#[derive(Error, Debug)]
pub enum StartupError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database connection failed: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    
    #[error("Tracing initialization failed: {0}")]
    TracingInit(String),
    
    #[error("Server bind failed: {0}")]
    ServerBind(String),
    
    #[error("gRPC server failed: {0}")]
    GrpcServer(String),
    
    #[error("Environment variable {0} not set")]
    EnvVarMissing(String),
}

impl From<std::env::VarError> for StartupError {
    fn from(e: std::env::VarError) -> Self {
        match e {
            std::env::VarError::NotPresent => StartupError::EnvVarMissing("Unknown".to_string()),
            std::env::VarError::NotUnicode(_) => StartupError::Configuration("Invalid Unicode in environment variable".to_string()),
        }
    }
}

impl From<std::io::Error> for StartupError {
    fn from(e: std::io::Error) -> Self {
        StartupError::ServerBind(e.to_string())
    }
}

pub type StartupResult<T> = Result<T, StartupError>; 