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
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                self.server_port = parsed_port;
            }
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".to_string());
        }

        Ok(())
    }
}