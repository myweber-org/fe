use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u32,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                self.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("APP_DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("APP_LOG_LEVEL") {
            self.log_level = log_level.to_lowercase();
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }
        
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".into());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config_parsing() {
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("APP_SERVER_PORT", "9090");
        env::set_var("APP_LOG_LEVEL", "DEBUG");
        
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("APP_SERVER_PORT");
        env::remove_var("APP_LOG_LEVEL");
    }
}use serde::{Deserialize, Serialize};
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
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
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
                max_connections: 20,
                min_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("app.log".to_string()),
            },
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections must be greater than or equal to min connections".to_string());
        }
        if self.database.max_connections == 0 {
            return Err("Max connections cannot be 0".to_string());
        }
        Ok(())
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
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("port = 8080"));
        assert!(toml_str.contains("max_connections = 20"));
    }

    #[test]
    fn test_config_from_file() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_str).unwrap();
        
        let loaded_config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_config.server.port, 8080);
    }
}