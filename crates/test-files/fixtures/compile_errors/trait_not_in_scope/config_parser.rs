use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u32,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                self.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("APP_DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("APP_LOG_LEVEL") {
            self.log_level = log_level.to_lowercase();
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }
        
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".into());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config_parsing() {
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("APP_SERVER_PORT", "9090");
        env::set_var("APP_LOG_LEVEL", "DEBUG");
        
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/mydb"
            log_level = "info"
            cache_ttl = 300
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("APP_SERVER_PORT");
        env::remove_var("APP_LOG_LEVEL");
    }
}