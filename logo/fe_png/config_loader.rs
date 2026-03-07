
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
}use serde::Deserialize;
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
        let config_str = fs::read_to_string(file_path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AppConfig {
            server_port: env::var("SERVER_PORT")?.parse()?,
            database_url: env::var("DATABASE_URL")?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            cache_ttl: env::var("CACHE_TTL")?.parse()?,
        })
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
            errors.push(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config = if let Ok(file_path) = env::var("CONFIG_FILE") {
        AppConfig::from_file(&file_path)?
    } else {
        AppConfig::from_env()?
    };

    config.validate()?;
    Ok(config)
}use std::collections::HashMap;
use std::env;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                values.insert(key.to_lowercase(), value);
            }
        }
        
        Config { values }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        let formatted_key = format!("app_{}", key.to_lowercase());
        self.values.get(&formatted_key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost:5432");
        env::set_var("APP_LOG_LEVEL", "debug");
        env::set_var("OTHER_VAR", "ignored");
        
        let config = Config::new();
        
        assert_eq!(config.get("database_url"), Some(&"postgres://localhost:5432".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
        assert_eq!(config.get("other_var"), None);
        assert_eq!(config.get_or_default("missing_key", "default_value"), "default_value");
    }
}