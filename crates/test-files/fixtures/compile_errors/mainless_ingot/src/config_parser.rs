use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
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
    #[serde(default = "default_database_name")]
    pub database: String,
    pub max_connections: Option<u32>,
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
    pub enable_compression: bool,
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub debug: bool,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
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
        if self.environment.is_empty() {
            return Err(ConfigError::MissingField("environment".to_string()));
        }

        if !["development", "staging", "production"].contains(&self.environment.as_str()) {
            return Err(ConfigError::InvalidValue {
                field: "environment".to_string(),
                value: self.environment.clone(),
            });
        }

        if self.database.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "database.port".to_string(),
                value: self.database.port.to_string(),
            });
        }

        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "server.port".to_string(),
                value: self.server.port.to_string(),
            });
        }

        Ok(())
    }

    pub fn server_url(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}@{}:{}/{}",
            self.database.username,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            debug: true,
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                database: default_database_name(),
                max_connections: Some(10),
            },
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: default_timeout(),
                enable_compression: false,
            },
            features: HashMap::new(),
        }
    }
}