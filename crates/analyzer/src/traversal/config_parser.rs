use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use toml;

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
    pub enable_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_size_mb: u64,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }

        if self.database.pool_size == 0 {
            errors.push("Database pool size cannot be zero".to_string());
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

    pub fn get_env_overrides(&self) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        
        if let Ok(host) = std::env::var("APP_SERVER_HOST") {
            overrides.insert("server.host".to_string(), host);
        }
        
        if let Ok(port) = std::env::var("APP_SERVER_PORT") {
            overrides.insert("server.port".to_string(), port);
        }
        
        if let Ok(db_host) = std::env::var("APP_DB_HOST") {
            overrides.insert("database.host".to_string(), db_host);
        }

        overrides
    }
}

pub fn load_config_with_fallback(default_path: &str, fallback_path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match AppConfig::from_file(default_path) {
        Ok(config) => Ok(config),
        Err(_) => AppConfig::from_file(fallback_path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                host: "db.local".to_string(),
                port: 5432,
                username: "admin".to_string(),
                password: "secret".to_string(),
                database_name: "appdb".to_string(),
                pool_size: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "/var/log/app.log".to_string(),
                max_size_mb: 100,
            },
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        config.to_file(path).unwrap();
        let loaded_config = AppConfig::from_file(path).unwrap();

        assert_eq!(loaded_config.server.host, "localhost");
        assert_eq!(loaded_config.server.port, 8080);
        assert_eq!(loaded_config.database.pool_size, 10);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 0,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                host: "db.local".to_string(),
                port: 5432,
                username: "admin".to_string(),
                password: "secret".to_string(),
                database_name: "appdb".to_string(),
                pool_size: 0,
            },
            logging: LoggingConfig {
                level: "invalid".to_string(),
                file_path: "/var/log/app.log".to_string(),
                max_size_mb: 100,
            },
        };

        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        if let Err(errors) = validation_result {
            assert!(errors.len() >= 3);
            assert!(errors.iter().any(|e| e.contains("port cannot be zero")));
            assert!(errors.iter().any(|e| e.contains("pool size cannot be zero")));
            assert!(errors.iter().any(|e| e.contains("Invalid log level")));
        }

        config.server.port = 8080;
        config.database.pool_size = 10;
        config.logging.level = "info".to_string();
        
        assert!(config.validate().is_ok());
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
                max_connections: 20,
                min_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_file_size_mb: 100,
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
                eprintln!("Failed to load config file: {}. Using defaults.", e);
                AppConfig::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections cannot be less than min connections".to_string());
        }
        if self.logging.max_file_size_mb == 0 {
            return Err("Max file size must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = self.to_toml()?;
        fs::write(path, toml_content)?;
        Ok(())
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
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
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
    fn test_config_file_operations() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        config.save_to_file(path).unwrap();
        let loaded_config = AppConfig::from_file(path).unwrap();
        
        assert_eq!(config.server.port, loaded_config.server.port);
        assert_eq!(config.database.url, loaded_config.database.url);
    }
}