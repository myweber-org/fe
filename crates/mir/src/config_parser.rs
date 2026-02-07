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
            cache_ttl: 300,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    
    Ok(config)
}

pub fn load_config_or_default<P: AsRef<Path>>(path: P) -> AppConfig {
    match load_config(path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}. Using defaults.", e);
            AppConfig::default()
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.database.port == 0 {
        return Err("Database port cannot be 0".to_string());
    }
    
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.pool_size == 0 {
        return Err("Database pool size cannot be 0".to_string());
    }
    
    if config.server.max_connections == 0 {
        return Err("Server max connections cannot be 0".to_string());
    }
    
    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.log_level.as_str()) {
        return Err(format!("Invalid log level: {}", config.log_level));
    }
    
    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = toml::to_string_pretty(config)?;
    fs::write(path, config_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_load_and_save_config() {
        let mut config = AppConfig::default();
        config.server.port = 9090;
        config.log_level = "debug".to_string();

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        save_config(&config, path).unwrap();
        let loaded_config = load_config(path).unwrap();

        assert_eq!(loaded_config.server.port, 9090);
        assert_eq!(loaded_config.log_level, "debug");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.database.port = 0;
        
        assert!(validate_config(&config).is_err());
    }
}