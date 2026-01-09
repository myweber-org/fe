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
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
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
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                enable_console: true,
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

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Configuration file not found or invalid, using defaults");
            Ok(AppConfig::default())
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections == 0 {
        return Err("Database max connections cannot be 0".to_string());
    }
    
    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = toml::to_string_pretty(config)?;
    fs::write(path, config_str)?;
    Ok(())
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureFlags {
    pub enable_cache: bool,
    pub enable_logging: bool,
    pub maintenance_mode: bool,
}

impl AppConfig {
    pub fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let mut config: AppConfig = serde_json::from_str(&config_content)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("APP_SERVER_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        
        if let Ok(max_conn) = env::var("DB_MAX_CONNECTIONS") {
            if let Ok(max_conn_num) = max_conn.parse::<u32>() {
                self.database.max_connections = max_conn_num;
            }
        }
        
        if let Ok(cache_enabled) = env::var("ENABLE_CACHE") {
            self.features.enable_cache = cache_enabled.to_lowercase() == "true";
        }
        
        if let Ok(maintenance) = env::var("MAINTENANCE_MODE") {
            self.features.maintenance_mode = maintenance.to_lowercase() == "true";
        }
    }
    
    pub fn merge_with(&mut self, other: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_value(self)?;
        let mut merged = serialized.clone();
        
        if let (Some(merged_obj), Some(other_obj)) = (merged.as_object_mut(), other.as_object()) {
            for (key, value) in other_obj {
                merged_obj.insert(key.clone(), value.clone());
            }
        }
        
        *self = serde_json::from_value(merged)?;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        if self.database.max_connections == 0 {
            errors.push("Database max connections cannot be 0".to_string());
        }
        
        if self.database.url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn load_config_with_fallback(config_path: &str, fallback_path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match AppConfig::load(config_path) {
        Ok(config) => Ok(config),
        Err(_) => {
            eprintln!("Primary config not found, using fallback: {}", fallback_path);
            AppConfig::load(fallback_path)
        }
    }
}