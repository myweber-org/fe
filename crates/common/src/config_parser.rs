use std::fs;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Database max connections cannot be zero".to_string());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.logging.level));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

pub fn load_config_with_fallback(path: &str, fallback_path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match AppConfig::from_file(path) {
        Ok(config) => Ok(config),
        Err(_) => AppConfig::from_file(fallback_path),
    }
}