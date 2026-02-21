
use serde::Deserialize;
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration value: {0}")]
    ValidationError(String),
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        
        Self::override_with_env(&mut config)?;
        Self::validate(&config)?;
        
        Ok(config)
    }
    
    fn override_with_env(config: &mut AppConfig) -> Result<(), ConfigError> {
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()
                .map_err(|_| ConfigError::ValidationError("Invalid port number".to_string()))?;
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            config.cache_ttl = cache_ttl.parse()
                .map_err(|_| ConfigError::ValidationError("Invalid cache TTL".to_string()))?;
        }
        
        Ok(())
    }
    
    fn validate(config: &AppConfig) -> Result<(), ConfigError> {
        if config.server_port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if config.database_url.is_empty() {
            return Err(ConfigError::ValidationError("Database URL cannot be empty".to_string()));
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}", config.log_level)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 300,
        };
        
        assert!(AppConfig::validate(&config).is_ok());
    }
    
    #[test]
    fn test_invalid_log_level() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "invalid".to_string(),
            cache_ttl: 300,
        };
        
        assert!(AppConfig::validate(&config).is_err());
    }
}