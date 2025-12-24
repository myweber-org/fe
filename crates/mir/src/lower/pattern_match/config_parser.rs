use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub timeout_seconds: u32,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
        
        Self::from_map(&config_map)
    }
    
    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config_map.insert(config_key, value);
            }
        }
        
        Self::from_map(&config_map)
    }
    
    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map.get("database_url")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or("Missing database_url configuration")?;
            
        let api_key = map.get("api_key")
            .map(|s| s.to_string())
            .or_else(|| env::var("API_KEY").ok())
            .ok_or("Missing api_key configuration")?;
            
        let debug_mode = map.get("debug_mode")
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false);
            
        let port = map.get("port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);
            
        let timeout_seconds = map.get("timeout_seconds")
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            timeout_seconds,
        })
    }
    
    pub fn load() -> Result<Self, String> {
        Self::from_file("config.ini")
            .or_else(|_| Self::from_env())
            .or_else(|_| Self::from_file("/etc/app/config.ini"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost/test");
        env::set_var("APP_API_KEY", "test-key-123");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "test-key-123");
        assert!(!config.debug_mode);
        assert_eq!(config.port, 8080);
    }
}