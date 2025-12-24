
use std::env;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid value for environment variable {0}: {1}")]
    InvalidValue(String, String),
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_tls: bool,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub feature_flags: HashMap<String, bool>,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = get_env_var("DB_HOST")?;
        let port = get_env_var("DB_PORT")?
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidValue("DB_PORT".to_string(), e.to_string()))?;
        
        let username = get_env_var("DB_USERNAME")?;
        let password = get_env_var("DB_PASSWORD")?;
        let database_name = get_env_var("DB_NAME")?;
        
        let max_connections = get_env_var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| ConfigError::InvalidValue("DB_MAX_CONNECTIONS".to_string(), e.to_string()))?;

        Ok(Self {
            host,
            port,
            username,
            password,
            database_name,
            max_connections,
        })
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.host.is_empty() {
            return Err(ConfigError::ValidationFailed("Database host cannot be empty".to_string()));
        }
        if self.port == 0 {
            return Err(ConfigError::ValidationFailed("Database port cannot be zero".to_string()));
        }
        if self.username.is_empty() {
            return Err(ConfigError::ValidationFailed("Database username cannot be empty".to_string()));
        }
        if self.max_connections == 0 {
            return Err(ConfigError::ValidationFailed("Max connections must be greater than zero".to_string()));
        }
        Ok(())
    }
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = get_env_var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = get_env_var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidValue("SERVER_PORT".to_string(), e.to_string()))?;
        
        let enable_tls = get_env_var("ENABLE_TLS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(|e| ConfigError::InvalidValue("ENABLE_TLS".to_string(), e.to_string()))?;
        
        let request_timeout_seconds = get_env_var("REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::InvalidValue("REQUEST_TIMEOUT".to_string(), e.to_string()))?;

        Ok(Self {
            host,
            port,
            enable_tls,
            request_timeout_seconds,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::ValidationFailed("Server port cannot be zero".to_string()));
        }
        if self.request_timeout_seconds == 0 {
            return Err(ConfigError::ValidationFailed("Request timeout must be greater than zero".to_string()));
        }
        Ok(())
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let database = DatabaseConfig::from_env()?;
        let server = ServerConfig::from_env()?;
        
        let log_level = get_env_var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        
        let mut feature_flags = HashMap::new();
        if let Ok(features) = get_env_var("FEATURE_FLAGS") {
            for flag in features.split(',') {
                let parts: Vec<&str> = flag.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let value = parts[1].trim().parse::<bool>().unwrap_or(false);
                    feature_flags.insert(key, value);
                }
            }
        }

        let config = Self {
            database,
            server,
            log_level,
            feature_flags,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        self.database.validate()?;
        self.server.validate()?;
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationFailed(
                format!("Invalid log level: {}. Must be one of: {}", 
                    self.log_level, valid_log_levels.join(", "))
            ));
        }
        
        Ok(())
    }

    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.feature_flags.get(feature_name).copied().unwrap_or(false)
    }
}

fn get_env_var(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_validation() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            database_name: "testdb".to_string(),
            max_connections: 10,
        };
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_database_config() {
        let config = DatabaseConfig {
            host: "".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            database_name: "testdb".to_string(),
            max_connections: 10,
        };
        
        assert!(config.validate().is_err());
    }
}