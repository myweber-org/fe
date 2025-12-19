use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_tls: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, String> {
    let config_str = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let mut config: ServerConfig = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    validate_config(&mut config)?;
    Ok(config)
}

fn validate_config(config: &mut ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be 0".to_string());
    }

    if config.max_connections == 0 {
        config.max_connections = 10;
        eprintln!("Warning: max_connections was 0, using default: 10");
    }

    if config.max_connections > 10000 {
        return Err("max_connections cannot exceed 10000".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_load_valid_config() {
        let config_str = r#"
            host = "0.0.0.0"
            port = 9000
            max_connections = 500
            enable_tls = true
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_str).unwrap();

        let config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 500);
        assert!(config.enable_tls);
    }

    #[test]
    fn test_invalid_port() {
        let config_str = r#"
            host = "localhost"
            port = 0
            max_connections = 100
            enable_tls = false
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_str).unwrap();

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Port cannot be 0"));
    }
}