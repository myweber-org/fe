
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

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
            .unwrap_or_else(|_| "config.json".to_string());

        if !Path::new(&config_path).exists() {
            return Err(format!("Configuration file not found: {}", config_path));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config: AppConfig = serde_json::from_str(&config_content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Self::apply_environment_overrides(&mut config);
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(config: &mut AppConfig) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                config.server_port = parsed_port;
            }
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_uppercase();
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
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let config_json = r#"{
            "server_port": 8080,
            "database_url": "postgres://localhost:5432/mydb",
            "log_level": "INFO",
            "cache_ttl": 300
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_json).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());

        let config = AppConfig::load();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.log_level, "INFO");
        assert_eq!(config.cache_ttl, 300);

        env::remove_var("CONFIG_PATH");
    }

    #[test]
    fn test_environment_overrides() {
        let config_json = r#"{
            "server_port": 8080,
            "database_url": "postgres://localhost:5432/mydb",
            "log_level": "INFO",
            "cache_ttl": 300
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_json).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        env::set_var("SERVER_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://prod:5432/proddb");
        env::set_var("LOG_LEVEL", "debug");

        let config = AppConfig::load().unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://prod:5432/proddb");
        assert_eq!(config.log_level, "DEBUG");

        env::remove_var("CONFIG_PATH");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
        env::remove_var("LOG_LEVEL");
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = r#"{
            "server_port": 0,
            "database_url": "",
            "log_level": "INVALID",
            "cache_ttl": 300
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_config).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());

        let result = AppConfig::load();
        assert!(result.is_err());

        env::remove_var("CONFIG_PATH");
    }
}