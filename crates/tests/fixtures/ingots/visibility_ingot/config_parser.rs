use std::collections::HashMap;
use std::fs;
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.server.workers == 0 {
            return Err("Number of workers cannot be zero".to_string());
        }
        
        if self.database.host.is_empty() {
            return Err("Database host cannot be empty".to_string());
        }
        
        Ok(())
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_config() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database = "appdb"
            
            [server]
            address = "0.0.0.0"
            port = 8080
            workers = 4
            
            [features]
            caching = true
            logging = false
        "#;
        
        let config: AppConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert!(config.is_feature_enabled("caching"));
        assert!(!config.is_feature_enabled("logging"));
    }
    
    #[test]
    fn test_invalid_config() {
        let toml_content = r#"
            [database]
            host = ""
            port = 5432
            username = "admin"
            password = "secret"
            database = "appdb"
            
            [server]
            address = "0.0.0.0"
            port = 8080
            workers = 4
        "#;
        
        let result: Result<AppConfig, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }
}