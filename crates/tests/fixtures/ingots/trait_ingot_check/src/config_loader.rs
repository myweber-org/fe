use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            cache_size: 1024,
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        if self.cache_size > 100_000 {
            return Err("Cache size too large".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_load_valid_config() {
        let yaml_content = r#"
server_port: 3000
database_url: "postgresql://localhost:5432/test_db"
log_level: "debug"
cache_size: 2048
"#;
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();
        
        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_invalid_config() {
        let yaml_content = r#"
server_port: 0
database_url: ""
log_level: "invalid"
cache_size: 200000
"#;
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();
        
        let result = AppConfig::load_from_file(temp_file.path());
        assert!(result.is_err());
    }
}