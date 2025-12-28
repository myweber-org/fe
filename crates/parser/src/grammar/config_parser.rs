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
    #[error("Invalid value for field {0}: {1}")]
    InvalidValue(String, String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_connections: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub enable_logging: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub rotation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub features: HashMap<String, bool>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
                max_connections: 100,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/appdb".to_string(),
                pool_size: 10,
                enable_logging: false,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                rotation: "daily".to_string(),
            },
            features: HashMap::from([
                ("caching".to_string(), true),
                ("metrics".to_string(), false),
                ("debug_endpoints".to_string(), false),
            ]),
        }
    }
}

impl ApplicationConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let mut config: ApplicationConfig = toml::from_str(&content)?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue(
                "server.port".to_string(),
                "Port cannot be zero".to_string()
            ));
        }
        
        if self.server.timeout_seconds > 3600 {
            return Err(ConfigError::InvalidValue(
                "server.timeout_seconds".to_string(),
                "Timeout cannot exceed 1 hour".to_string()
            ));
        }
        
        if self.database.pool_size == 0 {
            return Err(ConfigError::InvalidValue(
                "database.pool_size".to_string(),
                "Pool size must be greater than zero".to_string()
            ));
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidValue(
                "logging.level".to_string(),
                format!("Must be one of: {}", valid_log_levels.join(", "))
            ));
        }
        
        Ok(())
    }
    
    pub fn merge_with_defaults(mut self) -> Self {
        let default = ApplicationConfig::default();
        
        if self.server.host.is_empty() {
            self.server.host = default.server.host;
        }
        
        if self.server.port == 0 {
            self.server.port = default.server.port;
        }
        
        for (key, value) in default.features {
            self.features.entry(key).or_insert(value);
        }
        
        self
    }
    
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = ApplicationConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.logging.level, "info");
        assert!(config.features.get("caching").unwrap());
    }
    
    #[test]
    fn test_from_valid_file() {
        let toml_content = r#"
            [server]
            host = "0.0.0.0"
            port = 9000
            timeout_seconds = 60
            max_connections = 200
            
            [database]
            url = "postgresql://prod:5432/proddb"
            pool_size = 20
            enable_logging = true
            
            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            rotation = "hourly"
            
            [features]
            caching = false
            metrics = true
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();
        
        let config = ApplicationConfig::from_file(file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.database.pool_size, 20);
        assert_eq!(config.logging.level, "debug");
        assert!(!config.features.get("caching").unwrap());
        assert!(config.features.get("metrics").unwrap());
    }
    
    #[test]
    fn test_validation() {
        let mut config = ApplicationConfig::default();
        config.server.port = 0;
        
        let result = config.validate();
        assert!(result.is_err());
        
        if let Err(ConfigError::InvalidValue(field, _)) = result {
            assert_eq!(field, "server.port");
        }
    }
    
    #[test]
    fn test_merge_with_defaults() {
        let mut config = ApplicationConfig::default();
        config.server.host = String::new();
        config.server.port = 0;
        config.features.remove("caching");
        
        let merged = config.merge_with_defaults();
        assert_eq!(merged.server.host, "127.0.0.1");
        assert_eq!(merged.server.port, 8080);
        assert!(merged.features.get("caching").unwrap());
    }
}