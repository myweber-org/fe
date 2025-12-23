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
                file_path: None,
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

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> AppConfig {
    match load_config(path) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Warning: Using default configuration");
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
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
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
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        
        assert!(validate_config(&config).is_err());
    }
    
    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
            [server]
            host = "0.0.0.0"
            port = 3000
            timeout_seconds = 60
            
            [database]
            url = "postgresql://prod:5432/appdb"
            max_connections = 50
            min_connections = 10
            
            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, config_str.as_bytes()).unwrap();
        
        let result = load_config(file.path());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "debug");
    }
}