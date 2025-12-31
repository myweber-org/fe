use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
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
    
    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let api_key = Self::get_value(map, "API_KEY")?;
        let debug_mode = Self::get_bool(map, "DEBUG_MODE").unwrap_or(false);
        let port = Self::get_u16(map, "PORT").unwrap_or(8080);
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }
    
    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
        env::var(key)
            .ok()
            .or_else(|| map.get(key).cloned())
            .ok_or_else(|| format!("Missing required configuration: {}", key))
    }
    
    fn get_bool(map: &HashMap<String, String>, key: &str) -> Option<bool> {
        map.get(key)
            .map(|v| v.to_lowercase())
            .map(|v| v == "true" || v == "1" || v == "yes")
    }
    
    fn get_u16(map: &HashMap<String, String>, key: &str) -> Option<u16> {
        map.get(key)
            .and_then(|v| v.parse().ok())
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.api_key.len() < 16 {
            return Err("API key must be at least 16 characters".to_string());
        }
        
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "API_KEY=secret_key_1234567890").unwrap();
        writeln!(temp_file, "DEBUG_MODE=true").unwrap();
        writeln!(temp_file, "PORT=3000").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "secret_key_1234567890");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
    }
    
    #[test]
    fn test_env_var_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        
        let mut map = HashMap::new();
        map.insert("DATABASE_URL".to_string(), "postgres://dev/db".to_string());
        map.insert("API_KEY".to_string(), "test_key".to_string());
        
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        
        env::remove_var("DATABASE_URL");
    }
}