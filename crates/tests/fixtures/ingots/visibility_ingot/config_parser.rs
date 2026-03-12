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
}use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let mut value = trimmed[equal_pos + 1..].trim().to_string();
                
                value = var_pattern.replace_all(&value, |caps: ®ex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| String::new())
                }).to_string();
                
                self.values.insert(key, value);
            }
        }
        
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            database_host=localhost
            database_port=5432
            # This is a comment
            api_timeout=30
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("database_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("database_port"), Some(&"5432".to_string()));
        assert_eq!(parser.get("api_timeout"), Some(&"30".to_string()));
        assert_eq!(parser.get("nonexistent"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            secret=${APP_SECRET}
            fallback=${NONEXISTENT_VAR}
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("secret"), Some(&"my_secret_key".to_string()));
        assert_eq!(parser.get("fallback"), Some(&"".to_string()));
    }
}