use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
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
                connection_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, String> {
    let config_str = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: AppConfig = serde_yaml::from_str(&config_str)
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

    if config.logging.max_file_size_mb == 0 {
        config.logging.max_file_size_mb = 50;
    }

    let valid_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_levels.contains(&config.logging.level.as_str()) {
        config.logging.level = "info".to_string();
    }

    Ok(())
}

pub fn save_default_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let default_config = AppConfig::default();
    let yaml = serde_yaml::to_string(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;

    fs::write(path, yaml)
        .map_err(|e| format!("Failed to write default config: {}", e))?;

    Ok(())
}