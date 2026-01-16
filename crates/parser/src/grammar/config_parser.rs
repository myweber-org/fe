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
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_pattern.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                self.values.insert(key, value.trim().to_string());
            }
        }
        
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let content = "HOST=localhost\nPORT=8080\nDEBUG=true\n";
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(parser.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        env::set_var("DB_HOST", "postgres");
        
        let mut parser = ConfigParser::new();
        let content = "PORT=${APP_PORT}\nDATABASE=${DB_HOST}\nMODE=production\n";
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get("PORT"), Some(&"3000".to_string()));
        assert_eq!(parser.get("DATABASE"), Some(&"postgres".to_string()));
        assert_eq!(parser.get("MODE"), Some(&"production".to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.conf");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "SERVER=api.example.com\nTIMEOUT=30\n# Comment line\n\nLOG_LEVEL=info").unwrap();
        
        let mut parser = ConfigParser::new();
        parser.parse_file(file_path.to_str().unwrap()).unwrap();
        
        assert_eq!(parser.get("SERVER"), Some(&"api.example.com".to_string()));
        assert_eq!(parser.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(parser.get("LOG_LEVEL"), Some(&"info".to_string()));
        assert!(!parser.contains_key("# Comment line"));
    }

    #[test]
    fn test_get_or_default() {
        let mut parser = ConfigParser::new();
        let content = "EXISTING_KEY=value\n";
        parser.parse_content(content).unwrap();
        
        assert_eq!(parser.get_or_default("EXISTING_KEY", "default"), "value");
        assert_eq!(parser.get_or_default("MISSING_KEY", "default_value"), "default_value");
    }
}