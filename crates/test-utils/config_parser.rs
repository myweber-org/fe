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
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_seconds: u64,
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
                max_connections: 100,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                pool_size: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, String> {
    let config_str = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: AppConfig = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&mut config)?;
    Ok(config)
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), String> {
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, config_str)
        .map_err(|e| format!("Failed to write config file: {}", e))
}

fn validate_config(config: &mut AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }

    if config.server.max_connections == 0 {
        config.server.max_connections = 10;
    }

    if config.database.pool_size == 0 {
        config.database.pool_size = 5;
    }

    if config.database.timeout_seconds == 0 {
        config.database.timeout_seconds = 10;
    }

    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.logging.level.to_lowercase().as_str()) {
        config.logging.level = "info".to_string();
    }

    if config.logging.max_file_size_mb == 0 {
        config.logging.max_file_size_mb = 50;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_size, 10);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(validate_config(&mut config).is_err());

        let mut config = AppConfig::default();
        config.logging.level = "invalid".to_string();
        assert!(validate_config(&mut config).is_ok());
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_save_and_load() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(save_config(&config, path).is_ok());
        let loaded_config = load_config(path).unwrap();
        assert_eq!(config.server.port, loaded_config.server.port);
    }
}