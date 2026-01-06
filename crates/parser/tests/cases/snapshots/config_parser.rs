use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.database.port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database.host.is_empty() {
            return Err("Database host cannot be empty".into());
        }
        
        if self.database.port == 0 {
            return Err("Database port must be greater than 0".into());
        }
        
        if self.server.port == 0 {
            return Err("Server port must be greater than 0".into());
        }
        
        if self.server.workers == 0 {
            return Err("Number of workers must be greater than 0".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}. Must be one of: {:?}", 
                self.log_level, valid_log_levels).into());
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
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
                max_connections: 100,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                pool_size: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        if self.database.pool_size == 0 {
            return Err("Database pool size must be greater than 0".to_string());
        }
        
        if self.logging.max_file_size_mb == 0 {
            return Err("Max log file size must be greater than 0".to_string());
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
        assert_eq!(config.database.pool_size, 10);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        config.save_to_file(temp_file.path()).unwrap();
        let loaded_config = AppConfig::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.server.host, loaded_config.server.host);
        assert_eq!(config.server.port, loaded_config.server.port);
    }
}