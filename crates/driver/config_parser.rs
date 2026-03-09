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
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn process_value(raw: &str) -> String {
        if raw.starts_with('$') {
            let var_name = &raw[1..];
            env::var(var_name).unwrap_or_else(|_| raw.to_string())
        } else {
            raw.to_string()
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }
}use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_logging: bool,
    pub log_level: String,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|_| ConfigError::FileNotFound(path.as_ref().to_string_lossy().to_string()))?;

        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let mut server_address = None;
        let mut port = None;
        let mut max_connections = None;
        let mut enable_logging = None;
        let mut log_level = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(format!("Invalid line: {}", trimmed)));
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "server_address" => server_address = Some(value.to_string()),
                "port" => {
                    port = Some(value.parse::<u16>().map_err(|e| {
                        ConfigError::ParseError(format!("Invalid port value: {}", e))
                    })?);
                }
                "max_connections" => {
                    max_connections = Some(value.parse::<usize>().map_err(|e| {
                        ConfigError::ParseError(format!("Invalid max_connections value: {}", e))
                    })?);
                }
                "enable_logging" => {
                    enable_logging = Some(value.parse::<bool>().map_err(|e| {
                        ConfigError::ParseError(format!("Invalid enable_logging value: {}", e))
                    })?);
                }
                "log_level" => log_level = Some(value.to_string()),
                _ => return Err(ConfigError::ParseError(format!("Unknown key: {}", key))),
            }
        }

        let config = AppConfig {
            server_address: server_address
                .ok_or_else(|| ConfigError::ValidationError("server_address is required".to_string()))?,
            port: port.ok_or_else(|| ConfigError::ValidationError("port is required".to_string()))?,
            max_connections: max_connections
                .ok_or_else(|| ConfigError::ValidationError("max_connections is required".to_string()))?,
            enable_logging: enable_logging
                .ok_or_else(|| ConfigError::ValidationError("enable_logging is required".to_string()))?,
            log_level: log_level
                .ok_or_else(|| ConfigError::ValidationError("log_level is required".to_string()))?,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be 0".to_string()));
        }

        if self.max_connections == 0 {
            return Err(ConfigError::ValidationError(
                "max_connections must be greater than 0".to_string(),
            ));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            )));
        }

        Ok(())
    }

    pub fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_logging: true,
            log_level: "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config_str = r#"
            server_address = 192.168.1.100
            port = 3000
            max_connections = 500
            enable_logging = true
            log_level = debug
        "#;

        let config = AppConfig::from_str(config_str).unwrap();
        assert_eq!(config.server_address, "192.168.1.100");
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.enable_logging, true);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_missing_field() {
        let config_str = r#"
            server_address = 192.168.1.100
            port = 3000
            max_connections = 500
            enable_logging = true
        "#;

        let result = AppConfig::from_str(config_str);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn test_invalid_port() {
        let config_str = r#"
            server_address = 192.168.1.100
            port = 0
            max_connections = 500
            enable_logging = true
            log_level = info
        "#;

        let result = AppConfig::from_str(config_str);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}