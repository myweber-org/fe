
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
    pub fn from_file_and_env(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(file_path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_content)?;

        if let Ok(port) = env::var("APP_PORT") {
            config.server_port = port.parse()?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
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

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
server_port: 8080
database_url: "postgres://localhost:5432/mydb"
log_level: "info"
cache_ttl: 300
"#;
        write!(temp_file, "{}", config_content).unwrap();

        env::set_var("APP_PORT", "9090");
        
        let config = AppConfig::from_file_and_env(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server_port: 0,
            database_url: "".to_string(),
            log_level: "invalid".to_string(),
            cache_ttl: 100,
        };

        let result = invalid_config.validate();
        assert!(result.is_err());
    }
}