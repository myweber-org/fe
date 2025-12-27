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
        let config_str = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config: AppConfig = toml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: {}, using default configuration", e);
                AppConfig::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be 0"));
        }

        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than 0"));
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
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
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.log_level = String::from("invalid");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_load_from_file() {
        let config_content = r#"
            server_address = "0.0.0.0"
            server_port = 3000
            max_connections = 500
            enable_logging = true
            log_level = "debug"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();

        let config = AppConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_address, "0.0.0.0");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.log_level, "debug");
    }
}