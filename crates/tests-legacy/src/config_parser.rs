
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                settings.insert(key, value);
            }
        }

        Self::from_map(&settings)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut settings = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                settings.insert(config_key, value);
            }
        }

        Self::from_map(&settings)
    }

    fn from_map(settings: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = settings
            .get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/mydb".to_string());

        let port = settings
            .get("port")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);

        let log_level = settings
            .get("log_level")
            .map(|s| s.to_string())
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());

        let cache_size = settings
            .get("cache_size")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("CACHE_SIZE").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(1000);

        Ok(Config {
            database_url,
            port,
            log_level,
            cache_size,
        })
    }

    pub fn merge(self, other: Self) -> Self {
        Config {
            database_url: other.database_url,
            port: other.port,
            log_level: other.log_level,
            cache_size: other.cache_size,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: "postgres://localhost:5432/mydb".to_string(),
            port: 8080,
            log_level: "info".to_string(),
            cache_size: 1000,
        }
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
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
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
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
        writeln!(file, "").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("MAX_CONNECTIONS"), Some(&"100".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("SECRET_KEY", "my_secret_value");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "API_KEY=$SECRET_KEY").unwrap();
        writeln!(file, "STATIC_VALUE=test123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("API_KEY"), Some(&"my_secret_value".to_string()));
        assert_eq!(config.get("STATIC_VALUE"), Some(&"test123".to_string()));
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
        return Err("Max connections cannot be less than min connections".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Max log file size must be greater than 0".to_string());
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
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, String> {
    let config_str = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: AppConfig = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    validate_config(&mut config)?;
    Ok(config)
}

fn validate_config(config: &mut AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }

    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections cannot be less than min connections".to_string());
    }

    if config.database.max_connections == 0 {
        return Err("Max connections must be greater than 0".to_string());
    }

    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!(
            "Invalid log level '{}'. Must be one of: {:?}",
            config.logging.level, valid_log_levels
        ));
    }

    if let Some(ref path) = config.logging.file_path {
        if path.is_empty() {
            config.logging.file_path = None;
        }
    }

    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), String> {
    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, toml_string)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}