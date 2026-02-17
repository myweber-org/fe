use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub timeout_seconds: u64,
    pub enable_tls: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

impl ServerConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.port == 0 {
            return Err("Port cannot be zero");
        }
        if self.max_connections == 0 {
            return Err("Max connections must be greater than zero");
        }
        if self.enable_tls {
            if self.cert_path.is_none() || self.key_path.is_none() {
                return Err("TLS requires both certificate and key paths");
            }
        }
        Ok(())
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            host = "127.0.0.1"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
            enable_tls = false
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        
        let config = ServerConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.timeout_seconds, 30);
        assert!(!config.enable_tls);
    }

    #[test]
    fn test_tls_config_validation() {
        let config = ServerConfig {
            host: "localhost".to_string(),
            port: 443,
            max_connections: 50,
            timeout_seconds: 15,
            enable_tls: true,
            cert_path: Some("/path/cert.pem".to_string()),
            key_path: Some("/path/key.pem".to_string()),
        };
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_tls_config() {
        let config = ServerConfig {
            host: "localhost".to_string(),
            port: 443,
            max_connections: 50,
            timeout_seconds: 15,
            enable_tls: true,
            cert_path: None,
            key_path: Some("/path/key.pem".to_string()),
        };
        
        assert!(config.validate().is_err());
    }
}