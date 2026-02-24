use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
    pub enable_metrics: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/appdb"),
            log_level: String::from("info"),
            cache_size: 1000,
            enable_metrics: false,
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

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err("Invalid log level".to_string());
        }
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let default = Self::default();
        if self.server_port == 0 {
            self.server_port = default.server_port;
        }
        if self.database_url.is_empty() {
            self.database_url = default.database_url;
        }
        if self.log_level.is_empty() {
            self.log_level = default.log_level;
        }
        if self.cache_size == 0 {
            self.cache_size = default.cache_size;
        }
        self
    }
}use std::fs;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Database max connections cannot be zero".to_string());
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

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080
            enable_ssl = false

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 10
            timeout_seconds = 30

            [logging]
            level = "info"
            file_path = "/var/log/app.log"
            enable_console = true
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 0,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/test".to_string(),
                max_connections: 5,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "invalid".to_string(),
                file_path: None,
                enable_console: true,
            },
        };

        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.contains("port cannot be zero")));
        assert!(errors.iter().any(|e| e.contains("Invalid log level")));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }
        
        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key) || env::var(key).is_ok()
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost/db".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
    }

    #[test]
    fn test_env_override() {
        env::set_var("TEST_KEY", "env_value");
        
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");
        
        assert_eq!(config.get("TEST_KEY"), Some("env_value".to_string()));
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING_KEY", "default_value"), "default_value");
    }
}