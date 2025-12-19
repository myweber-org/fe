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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.apply_env_overrides();
        Ok(config)
    }
    
    fn apply_env_overrides(&mut self) {
        if let Ok(port) = env::var("APP_PORT") {
            if let Ok(parsed) = port.parse() {
                self.server_port = parsed;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
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
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let toml_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }
    
    #[test]
    fn test_env_override() {
        env::set_var("APP_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://prod/db");
        
        let toml_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://prod/db");
        
        env::remove_var("APP_PORT");
        env::remove_var("DATABASE_URL");
    }
}