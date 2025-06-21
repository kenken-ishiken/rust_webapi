use serde::Deserialize;
use std::env;
use crate::infrastructure::startup_error::{StartupError, StartupResult};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub http_host: String,
    pub http_port: u16,
    pub grpc_host: String,
    pub grpc_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub keycloak_realm: String,
    pub keycloak_auth_server_url: String,
    pub keycloak_client_id: String,
}



impl AppConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> StartupResult<Self> {
        Ok(Self {
            database: DatabaseConfig::from_env()?,
            server: ServerConfig::from_env()?,
            auth: AuthConfig::from_env()?,
        })
    }

    /// 設定の検証を行う
    pub fn validate(&self) -> StartupResult<()> {
        // データベース設定の検証
        if self.database.url.is_empty() {
            return Err(StartupError::Configuration("Database URL cannot be empty".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(StartupError::Configuration("Max connections must be greater than 0".to_string()));
        }

        // サーバー設定の検証
        if self.server.http_port == 0 || self.server.grpc_port == 0 {
            return Err(StartupError::Configuration("Port numbers must be greater than 0".to_string()));
        }

        // Auth設定の検証
        if self.auth.keycloak_realm.is_empty() {
            return Err(StartupError::Configuration("Keycloak realm cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl DatabaseConfig {
    fn from_env() -> StartupResult<Self> {
        Ok(Self {
            url: env::var("DATABASE_URL")
                .map_err(|_| StartupError::EnvVarMissing("DATABASE_URL".to_string()))?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| StartupError::Configuration("Invalid DATABASE_MAX_CONNECTIONS".to_string()))?,
        })
    }
}

impl ServerConfig {
    fn from_env() -> StartupResult<Self> {
        Ok(Self {
            http_host: env::var("HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            http_port: env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| StartupError::Configuration("Invalid HTTP_PORT".to_string()))?,
            grpc_host: env::var("GRPC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .map_err(|_| StartupError::Configuration("Invalid GRPC_PORT".to_string()))?,
        })
    }
}

impl AuthConfig {
    fn from_env() -> StartupResult<Self> {
        Ok(Self {
            keycloak_realm: env::var("KEYCLOAK_REALM")
                .map_err(|_| StartupError::EnvVarMissing("KEYCLOAK_REALM".to_string()))?,
            keycloak_auth_server_url: env::var("KEYCLOAK_AUTH_SERVER_URL")
                .map_err(|_| StartupError::EnvVarMissing("KEYCLOAK_AUTH_SERVER_URL".to_string()))?,
            keycloak_client_id: env::var("KEYCLOAK_CLIENT_ID")
                .map_err(|_| StartupError::EnvVarMissing("KEYCLOAK_CLIENT_ID".to_string()))?,
        })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_server_config_defaults() {
        // 環境変数をクリア
        env::remove_var("HTTP_HOST");
        env::remove_var("HTTP_PORT");
        env::remove_var("GRPC_HOST");
        env::remove_var("GRPC_PORT");

        let config = ServerConfig::from_env().expect("Should use defaults");
        
        assert_eq!(config.http_host, "127.0.0.1");
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.grpc_host, "127.0.0.1");
        assert_eq!(config.grpc_port, 50051);
    }

    #[test]
    fn test_database_config_validation() {
        env::set_var("DATABASE_URL", "");
        
        let database_config = DatabaseConfig {
            url: "".to_string(),
            max_connections: 5,
        };
        
        let config = AppConfig {
            database: database_config,
            server: ServerConfig {
                http_host: "127.0.0.1".to_string(),
                http_port: 8080,
                grpc_host: "127.0.0.1".to_string(),
                grpc_port: 50051,
            },
            auth: AuthConfig {
                keycloak_realm: "test".to_string(),
                keycloak_auth_server_url: "http://localhost:8080".to_string(),
                keycloak_client_id: "test-client".to_string(),
            },
        };
        
        assert!(config.validate().is_err());
    }
} 