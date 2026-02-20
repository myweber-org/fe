use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_size_mb: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&content)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("APP_SERVER_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        
        if self.database.max_connections == 0 {
            errors.push("Database max connections cannot be zero".to_string());
        }
        
        if self.logging.max_size_mb == 0 {
            errors.push("Log max size cannot be zero".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let yaml_content = r#"
server:
  host: "localhost"
  port: 8080
  tls_enabled: false
database:
  url: "postgresql://localhost/mydb"
  max_connections: 10
  timeout_seconds: 30
logging:
  level: "info"
  file_path: "/var/log/app.log"
  max_size_mb: 100
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, yaml_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("APP_SERVER_HOST", "0.0.0.0");
        env::set_var("LOG_LEVEL", "debug");
        
        let yaml_content = r#"
server:
  host: "localhost"
  port: 8080
  tls_enabled: false
database:
  url: "postgresql://localhost/mydb"
  max_connections: 10
  timeout_seconds: 30
logging:
  level: "info"
  file_path: "/var/log/app.log"
  max_size_mb: 100
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, yaml_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.logging.level, "debug");
        
        env::remove_var("APP_SERVER_HOST");
        env::remove_var("LOG_LEVEL");
    }
}