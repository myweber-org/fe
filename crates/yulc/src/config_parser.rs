use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;

        let parsed: HashMap<String, toml::Value> = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Self::validate_and_build(parsed)
    }

    fn validate_and_build(data: HashMap<String, toml::Value>) -> Result<Self, ConfigError> {
        let database = Self::parse_database(&data)?;
        let server = Self::parse_server(&data)?;
        let features = Self::parse_features(&data);

        Ok(Config {
            database,
            server,
            features,
        })
    }

    fn parse_database(data: &HashMap<String, toml::Value>) -> Result<DatabaseConfig, ConfigError> {
        let db_table = data.get("database")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing database section".to_string()))?;

        let host = db_table.get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.host".to_string()))?
            .to_string();

        let port = db_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.port".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError("Invalid database port".to_string()));
        }

        let username = db_table.get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.username".to_string()))?
            .to_string();

        let password = db_table.get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing database.password".to_string()))?
            .to_string();

        Ok(DatabaseConfig {
            host,
            port: port as u16,
            username,
            password,
        })
    }

    fn parse_server(data: &HashMap<String, toml::Value>) -> Result<ServerConfig, ConfigError> {
        let server_table = data.get("server")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing server section".to_string()))?;

        let host = server_table.get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConfigError::ValidationError("Missing server.host".to_string()))?
            .to_string();

        let port = server_table.get("port")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| ConfigError::ValidationError("Missing server.port".to_string()))?;

        if port < 1 || port > 65535 {
            return Err(ConfigError::ValidationError("Invalid server port".to_string()));
        }

        let enable_ssl = server_table.get("enable_ssl")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(ServerConfig {
            host,
            port: port as u16,
            enable_ssl,
        })
    }

    fn parse_features(data: &HashMap<String, toml::Value>) -> HashMap<String, bool> {
        data.get("features")
            .and_then(|v| v.as_table())
            .map(|table| {
                table.iter()
                    .filter_map(|(key, value)| value.as_bool().map(|b| (key.clone(), b)))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"

            [server]
            host = "0.0.0.0"
            port = 8080
            enable_ssl = true

            [features]
            logging = true
            metrics = false
        "#;

        let temp_file = "test_config.toml";
        fs::write(temp_file, toml_content).unwrap();

        let config = Config::from_file(temp_file).unwrap();

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_ssl);
        assert_eq!(config.features.get("logging"), Some(&true));
        assert_eq!(config.features.get("metrics"), Some(&false));

        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_missing_section() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
        "#;

        let temp_file = "test_missing.toml";
        fs::write(temp_file, toml_content).unwrap();

        let result = Config::from_file(temp_file);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));

        fs::remove_file(temp_file).unwrap();
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
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections must be greater than or equal to min connections".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Max file size must be greater than 0".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}. Must be one of: {:?}", 
            config.logging.level, valid_log_levels));
    }
    
    Ok(())
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
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
            [server]
            host = "0.0.0.0"
            port = 3000
            timeout_seconds = 60
            
            [database]
            url = "postgresql://localhost:5432/testdb"
            max_connections = 15
            min_connections = 5
            
            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            max_file_size_mb = 50
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();
        
        let result = load_config(temp_file.path());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "debug");
    }
    
    #[test]
    fn test_validate_config_invalid_port() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port cannot be 0"));
    }
    
    #[test]
    fn test_validate_config_invalid_log_level() {
        let mut config = AppConfig::default();
        config.logging.level = "invalid".to_string();
        
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid log level"));
    }
}