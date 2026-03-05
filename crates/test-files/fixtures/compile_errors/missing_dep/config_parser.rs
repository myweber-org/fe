use std::fs;
use std::collections::HashMap;
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Database max connections must be greater than zero".to_string());
        }

        if !["error", "warn", "info", "debug", "trace"].contains(&self.logging.level.as_str()) {
            errors.push(format!("Invalid logging level: {}", self.logging.level));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DATABASE_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080
            timeout_seconds = 30

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 10
            pool_timeout = 5

            [logging]
            level = "info"
            file_path = "/var/log/app.log"
            enable_console = true
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 0,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/test".to_string(),
                max_connections: 5,
                pool_timeout: 5,
            },
            logging: LoggingConfig {
                level: "invalid".to_string(),
                file_path: None,
                enable_console: true,
            },
        };

        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.contains("port cannot be zero")));
        assert!(errors.iter().any(|e| e.contains("Invalid logging level")));
    }
}