use std::collections::HashMap;
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

pub fn load_config() -> Config {
    Config::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost:5432");
        env::set_var("APP_API_KEY", "secret123");
        
        let config = Config::new();
        
        assert_eq!(config.get("database_url"), Some(&"postgres://localhost:5432".to_string()));
        assert_eq!(config.get("api_key"), Some(&"secret123".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }
    
    #[test]
    fn test_get_or_default() {
        let config = Config::new();
        
        assert_eq!(config.get_or_default("missing_key", "default_value"), "default_value");
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
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_FILE")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_uppercase();
        }

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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            server_port = 8080
            database_url = "postgres://localhost/db"
            log_level = "INFO"
            cache_ttl = 300
            "#
        )
        .unwrap();

        env::set_var("CONFIG_FILE", file.path().to_str().unwrap());
        let config = AppConfig::from_env().unwrap();

        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.log_level, "INFO");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_env_override() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            server_port = 8080
            database_url = "postgres://localhost/db"
            log_level = "INFO"
            cache_ttl = 300
            "#
        )
        .unwrap();

        env::set_var("CONFIG_FILE", file.path().to_str().unwrap());
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "debug");

        let config = AppConfig::from_env().unwrap();

        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "DEBUG");
    }

    #[test]
    fn test_validation() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            server_port = 0
            database_url = ""
            log_level = "INVALID"
            cache_ttl = 300
            "#
        )
        .unwrap();

        env::set_var("CONFIG_FILE", file.path().to_str().unwrap());
        let result = AppConfig::from_env();

        assert!(result.is_err());
    }
}