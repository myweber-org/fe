use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub server_port: u16,
    pub max_connections: usize,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_address: String::from("127.0.0.1"),
            server_port: 8080,
            max_connections: 100,
            enable_logging: true,
            log_level: String::from("info"),
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Using default configuration. Error: {}", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        
        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than zero"));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }

    pub fn server_endpoint(&self) -> String {
        format!("{}:{}", self.server_address, self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.enable_logging, true);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_endpoint() {
        let config = AppConfig::default();
        assert_eq!(config.server_endpoint(), "127.0.0.1:8080");
    }

    #[test]
    fn test_load_from_file() {
        let toml_content = r#"
            server_address = "192.168.1.100"
            server_port = 9000
            max_connections = 250
            enable_logging = true
            log_level = "debug"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_address, "192.168.1.100");
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.max_connections, 250);
        assert_eq!(config.log_level, "debug");
    }
}