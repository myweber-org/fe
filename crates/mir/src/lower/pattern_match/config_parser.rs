use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub timeout_seconds: u32,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
        
        Self::from_map(&config_map)
    }
    
    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config_map.insert(config_key, value);
            }
        }
        
        Self::from_map(&config_map)
    }
    
    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map.get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or("Missing database_url configuration")?;
            
        let api_key = map.get("api_key")
            .map(|s| s.to_string())
            .or_else(|| env::var("API_KEY").ok())
            .ok_or("Missing api_key configuration")?;
            
        let debug_mode = map.get("debug_mode")
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false);
            
        let port = map.get("port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);
            
        let timeout_seconds = map.get("timeout_seconds")
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            timeout_seconds,
        })
    }
    
    pub fn load() -> Result<Self, String> {
        Self::from_file("config.ini")
            .or_else(|_| Self::from_env())
            .or_else(|_| Self::from_file("/etc/app/config.ini"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost/test");
        env::set_var("APP_API_KEY", "test-key-123");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "test-key-123");
        assert!(!config.debug_mode);
        assert_eq!(config.port, 8080);
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

pub fn load_config(config_path: &str) -> Result<AppConfig, String> {
    let path = Path::new(config_path);
    
    if !path.exists() {
        return Err(format!("Configuration file not found: {}", config_path));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&config)?;
    
    Ok(config)
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
    
    Ok(())
}

pub fn save_default_config(config_path: &str) -> Result<(), String> {
    let default_config = AppConfig::default();
    let toml_string = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    
    fs::write(config_path, toml_string)
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
    
    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "  TIMEOUT = 30  ").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=${APP_SECRET}").unwrap();
        writeln!(file, "OTHER_KEY=static_value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY"), Some(&"my_secret_key".to_string()));
        assert_eq!(config.get("OTHER_KEY"), Some(&"static_value".to_string()));
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}