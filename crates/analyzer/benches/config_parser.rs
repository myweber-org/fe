use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub enable_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub environment: String,
    pub debug_mode: bool,
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::MissingField("database.host".to_string()));
        }
        
        if self.database.port == 0 {
            return Err(ConfigError::ValidationFailed("Database port cannot be zero".to_string()));
        }
        
        if self.server.port == 0 {
            return Err(ConfigError::ValidationFailed("Server port cannot be zero".to_string()));
        }
        
        if self.environment.is_empty() {
            return Err(ConfigError::MissingField("environment".to_string()));
        }
        
        Ok(())
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, ConfigError> {
    let path_ref = path.as_ref();
    
    if !path_ref.exists() {
        return Err(ConfigError::FileNotFound(
            path_ref.to_string_lossy().to_string()
        ));
    }
    
    let content = fs::read_to_string(path_ref)
        .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    
    let config: AppConfig = serde_yaml::from_str(&content)
        .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    
    config.validate()?;
    
    Ok(config)
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), ConfigError> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    
    fs::write(path, yaml)
        .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config_loading() {
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: admin
  database: app_db
  max_connections: 10
server:
  address: 0.0.0.0
  port: 8080
  timeout_seconds: 30
  enable_logging: true
environment: production
debug_mode: false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let result = load_config(temp_file.path());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.environment, "production");
    }

    #[test]
    fn test_missing_field_validation() {
        let config_yaml = r#"
database:
  host: ""
  port: 5432
  username: admin
  database: app_db
  max_connections: 10
server:
  address: 0.0.0.0
  port: 8080
  timeout_seconds: 30
  enable_logging: true
environment: production
debug_mode: false
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let result = load_config(temp_file.path());
        assert!(result.is_err());
        
        if let Err(ConfigError::MissingField(field)) = result {
            assert_eq!(field, "database.host");
        } else {
            panic!("Expected MissingField error");
        }
    }
}