
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut values = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }
            
            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();
            
            value = Self::substitute_env_vars(&value);
            values.insert(key, value);
        }
        
        Ok(Config { values })
    }
    
    fn substitute_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        
        for (key, value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &value);
        }
        
        result
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub feature_flags: HashMap<String, bool>,
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
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            config_map.insert(key, value);
        }
        
        Self::from_map(config_map)
    }
    
    fn from_map(mut map: HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(&mut map, "DATABASE_URL")?;
        let port_str = Self::get_value(&mut map, "PORT")?;
        let port = port_str.parse::<u16>()
            .map_err(|e| format!("Invalid port number: {}", e))?;
        
        let log_level = Self::get_value(&mut map, "LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
        let mut feature_flags = HashMap::new();
        for (key, value) in map {
            if key.starts_with("FEATURE_") {
                let flag_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let flag_value = value.parse::<bool>()
                    .map_err(|e| format!("Invalid boolean for {}: {}", key, e))?;
                feature_flags.insert(flag_name, flag_value);
            }
        }
        
        Ok(Config {
            database_url,
            port,
            log_level,
            feature_flags,
        })
    }
    
    fn get_value(map: &mut HashMap<String, String>, key: &str) -> Result<String, String> {
        if let Some(value) = env::var(key).ok() {
            return Ok(value);
        }
        
        map.remove(key)
            .ok_or_else(|| format!("Missing required configuration: {}", key))
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "FEATURE_API_V2=true").unwrap();
        writeln!(temp_file, "FEATURE_CACHE=false").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.is_feature_enabled("api_v2"));
        assert!(!config.is_feature_enabled("cache"));
    }
    
    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgres://env/test");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://file/test").unwrap();
        writeln!(temp_file, "PORT=9090").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://env/test");
        
        env::remove_var("DATABASE_URL");
    }
}