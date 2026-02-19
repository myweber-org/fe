use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            timeout_seconds: 30,
            enable_logging: true,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: ServerConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(ServerConfig::default())
        }
    }
}

fn validate_config(config: &ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err(String::from("Port cannot be zero"));
    }
    
    if config.timeout_seconds > 3600 {
        return Err(String::from("Timeout cannot exceed one hour"));
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
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.enable_logging);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = ServerConfig {
            host: String::from("localhost"),
            port: 3000,
            timeout_seconds: 60,
            enable_logging: false,
        };
        
        assert!(validate_config(&valid_config).is_ok());
        
        let invalid_port_config = ServerConfig {
            port: 0,
            ..valid_config.clone()
        };
        
        assert!(validate_config(&invalid_port_config).is_err());
    }

    #[test]
    fn test_load_config_from_file() {
        let toml_content = r#"
            host = "0.0.0.0"
            port = 9000
            timeout_seconds = 120
            enable_logging = false
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
        assert_eq!(config.timeout_seconds, 120);
        assert!(!config.enable_logging);
    }
}