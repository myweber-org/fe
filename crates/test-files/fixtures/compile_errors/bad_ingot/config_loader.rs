use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(parsed_port) = port.parse() {
                self.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("APP_DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("APP_LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server_port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.log_level));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}