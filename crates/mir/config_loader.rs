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
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours (86400 seconds)".to_string());
        }

        Ok(())
    }

    pub fn get_server_address(&self) -> String {
        format!("0.0.0.0:{}", self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "INFO".to_string(),
            cache_ttl: 3600,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_overrides() {
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "debug");

        let mut config = AppConfig {
            server_port: 8080,
            database_url: "test".to_string(),
            log_level: "INFO".to_string(),
            cache_ttl: 3600,
        };

        config.apply_environment_overrides();

        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "DEBUG");

        env::remove_var("SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }
}