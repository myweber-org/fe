use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub enable_cache: bool,
    pub cache_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            enable_cache: true,
            cache_size: 100,
        }
    }
}

impl AppConfig {
    pub fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(config_path);
        
        if !path.exists() {
            println!("Config file not found at {}, using defaults", config_path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        if self.cache_size > 10000 {
            return Err("Cache size cannot exceed 10000".to_string());
        }
        
        Ok(())
    }
    
    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(self)?)
    }
}