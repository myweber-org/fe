use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub numeric_values: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            numeric_values: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if let Ok(num) = value.parse::<f64>() {
                config.numeric_values.insert(key.clone(), num);
            } else if value == "true" || value == "false" {
                let flag = value == "true";
                config.flags.insert(key.clone(), flag);
            } else {
                config.settings.insert(key, value);
            }
        }
        
        Ok(config)
    }
    
    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
    
    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }
    
    pub fn get_flag(&self, key: &str) -> Option<bool> {
        self.flags.get(key).copied()
    }
    
    pub fn get_setting_with_default(&self, key: &str, default: &str) -> String {
        self.get_setting(key)
            .map(|s| s.clone())
            .unwrap_or_else(|| default.to_string())
    }
    
    pub fn get_numeric_with_default(&self, key: &str, default: f64) -> f64 {
        self.get_numeric(key).unwrap_or(default)
    }
    
    pub fn get_flag_with_default(&self, key: &str, default: bool) -> bool {
        self.get_flag(key).unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Sample configuration").unwrap();
        writeln!(file, "app_name = MyApplication").unwrap();
        writeln!(file, "max_connections = 100").unwrap();
        writeln!(file, "debug_mode = true").unwrap();
        writeln!(file, "timeout = 30.5").unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        
        assert_eq!(config.get_setting("app_name"), Some(&"MyApplication".to_string()));
        assert_eq!(config.get_numeric("max_connections"), Some(100.0));
        assert_eq!(config.get_flag("debug_mode"), Some(true));
        assert_eq!(config.get_numeric("timeout"), Some(30.5));
        assert_eq!(config.get_setting("nonexistent"), None);
    }
    
    #[test]
    fn test_default_values() {
        let config = Config::new();
        
        assert_eq!(config.get_setting_with_default("missing", "default_value"), "default_value");
        assert_eq!(config.get_numeric_with_default("missing", 42.0), 42.0);
        assert_eq!(config.get_flag_with_default("missing", true), true);
    }
    
    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid_line_without_equals").unwrap();
        
        let result = Config::from_file(file.path());
        assert!(result.is_err());
    }
}use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_content)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(db_host) = env::var("DB_HOST") {
            self.database.host = db_host;
        }
        
        if let Ok(server_port) = env::var("SERVER_PORT") {
            if let Ok(port) = server_port.parse() {
                self.server.port = port;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.database.port == 0 {
            return Err("Database port cannot be 0".to_string());
        }
        
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 8080
  enable_https: false
  max_connections: 100

log_level: info
cache_ttl: 300
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("DB_HOST", "prod-db.example.com");
        env::set_var("LOG_LEVEL", "debug");
        
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 8080
  enable_https: false
  max_connections: 100

log_level: info
cache_ttl: 300
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "prod-db.example.com");
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("DB_HOST");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation() {
        let invalid_config_yaml = r#"
database:
  host: localhost
  port: 0
  username: postgres
  password: secret
  database_name: myapp

server:
  address: 0.0.0.0
  port: 8080
  enable_https: false
  max_connections: 100

log_level: invalid_level
cache_ttl: 300
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_config_yaml).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}