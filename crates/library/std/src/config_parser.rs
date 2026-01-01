use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config(path: &str) -> Result<AppConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    validate_config(&mut config)?;
    Ok(config)
}

fn validate_config(config: &mut AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }

    if config.database.max_connections == 0 {
        config.database.max_connections = 5;
    }

    if config.logging.max_file_size_mb > 1024 {
        return Err("Max log file size cannot exceed 1024 MB".to_string());
    }

    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        config.logging.level = "info".to_string();
    }

    Ok(())
}

pub fn generate_default_config(path: &str) -> Result<(), String> {
    let default_config = AppConfig::default();
    let toml_string = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;

    fs::write(path, toml_string)
        .map_err(|e| format!("Failed to write default config: {}", e))?;

    Ok(())
}

pub fn merge_configs(base: &AppConfig, overrides: &HashMap<String, String>) -> AppConfig {
    let mut merged = base.clone();

    for (key, value) in overrides {
        match key.as_str() {
            "server.host" => merged.server.host = value.clone(),
            "server.port" => if let Ok(port) = value.parse() { merged.server.port = port },
            "server.timeout_seconds" => if let Ok(timeout) = value.parse() { merged.server.timeout_seconds = timeout },
            "database.url" => merged.database.url = value.clone(),
            "database.max_connections" => if let Ok(conns) = value.parse() { merged.database.max_connections = conns },
            "logging.level" => merged.logging.level = value.clone(),
            "logging.file_path" => merged.logging.file_path = Some(value.clone()),
            "logging.max_file_size_mb" => if let Ok(size) = value.parse() { merged.logging.max_file_size_mb = size },
            _ => {}
        }
    }

    merged
}