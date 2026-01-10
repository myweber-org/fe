use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_ssl: bool,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "".to_string(),
                database_name: "app_db".to_string(),
                pool_size: 10,
            },
            server: ServerConfig {
                address: "0.0.0.0".to_string(),
                port: 8080,
                enable_ssl: false,
                max_connections: 100,
            },
            log_level: "info".to_string(),
            cache_ttl: 3600,
        }
    }
}

pub fn load_config(config_path: &str) -> Result<AppConfig, String> {
    let path = Path::new(config_path);
    
    if !path.exists() {
        return Err(format!("Configuration file not found: {}", config_path));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&config)?;
    
    Ok(config)
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.pool_size == 0 {
        return Err("Database pool size cannot be 0".to_string());
    }
    
    if config.cache_ttl > 86400 {
        return Err("Cache TTL cannot exceed 24 hours".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.log_level.as_str()) {
        return Err(format!("Invalid log level: {}", config.log_level));
    }
    
    Ok(())
}

pub fn save_default_config(config_path: &str) -> Result<(), String> {
    let default_config = AppConfig::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    
    fs::write(config_path, toml_content)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        assert!(save_default_config(path).is_ok());
        assert!(load_config(path).is_ok());
    }
}