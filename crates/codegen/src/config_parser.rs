use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
    pub enable_metrics: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/appdb"),
            log_level: String::from("info"),
            cache_size: 1000,
            enable_metrics: false,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err("Invalid log level".to_string());
        }
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let default = Self::default();
        if self.server_port == 0 {
            self.server_port = default.server_port;
        }
        if self.database_url.is_empty() {
            self.database_url = default.database_url;
        }
        if self.log_level.is_empty() {
            self.log_level = default.log_level;
        }
        if self.cache_size == 0 {
            self.cache_size = default.cache_size;
        }
        self
    }
}