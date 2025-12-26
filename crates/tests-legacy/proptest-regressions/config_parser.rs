use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for {field}: {value}")]
    InvalidValue {
        field: String,
        value: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_database_name")]
    pub database: String,
}

fn default_database_name() -> String {
    "app_db".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub enable_compression: bool,
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub file_path: Option<String>,
    #[serde(default)]
    pub enable_console: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.port".to_string(),
                value: self.server.port.to_string(),
            });
        }

        if self.database.host.is_empty() {
            return Err(ConfigError::MissingField("database.host".to_string()));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "logging.level".to_string(),
                value: self.logging.level.clone(),
            });
        }

        Ok(())
    }

    pub fn default_config() -> Self {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: None,
                database: default_database_name(),
            },
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: default_timeout(),
                enable_compression: false,
            },
            logging: LoggingConfig {
                level: default_log_level(),
                file_path: None,
                enable_console: true,
            },
            features: HashMap::new(),
        }
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default_config();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default_config();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default_config();
        let toml_str = config.to_toml();
        assert!(toml_str.is_ok());
    }
}